//! Circuit breaker — the canonical reliability FSM.
//!
//! A wrapper around a fallible dependency that stops hammering it when it's
//! failing. Three states:
//!
//! * **Closed** — traffic flows normally; track consecutive failures. Trip
//!   to `Open` after `failure_threshold` failures in a row.
//! * **Open** — reject everything immediately (no calls to the dependency).
//!   After `cooldown_steps` idle steps, move to `HalfOpen`.
//! * **HalfOpen** — allow exactly one probe call. Success closes the circuit;
//!   failure re-opens it.
//!
//! Implemented in Netflix Hystrix, resilience4j, Polly, every sensible
//! microservice client.
//!
//! This example feeds a scripted sequence of call outcomes and prints the
//! breaker's decisions alongside its state.

use state_machines_rs::{Runner, StateMachine};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallOutcome {
    Success,
    Failure,
    /// No request arrived this tick (idle).
    Idle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    /// Pass the call through.
    Allow,
    /// Reject without calling the dependency.
    Reject,
    /// No request to decide on.
    Idle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BreakerState {
    Closed { consecutive_failures: u32 },
    /// `cooldown_remaining` is the number of steps left before probing.
    Open { cooldown_remaining: u32 },
    HalfOpen,
}

pub struct CircuitBreaker {
    failure_threshold: u32,
    cooldown_steps: u32,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, cooldown_steps: u32) -> Self {
        Self { failure_threshold, cooldown_steps }
    }
}

impl StateMachine for CircuitBreaker {
    type Input = CallOutcome;
    /// `(decision_for_THIS_call, resulting_state)` so downstream logic can
    /// both act and log.
    type Output = (Decision, BreakerState);
    type State = BreakerState;

    fn start_state(&self) -> BreakerState {
        BreakerState::Closed { consecutive_failures: 0 }
    }

    fn next_values(&self, state: &BreakerState, outcome: &CallOutcome) -> (BreakerState, Self::Output) {
        use BreakerState::*;
        use CallOutcome::*;

        // Decision reflects the ACTION taken this step — for a real call it
        // comes from the current state before applying the outcome.
        let decision = match (state, outcome) {
            (_, Idle) => Decision::Idle,
            (Closed { .. }, _) => Decision::Allow,
            (HalfOpen, _) => Decision::Allow, // the probe call is allowed
            (Open { .. }, _) => Decision::Reject,
        };

        let next = match (*state, *outcome) {
            // Closed: track failures, trip after threshold.
            (Closed { .. }, Success) => Closed { consecutive_failures: 0 },
            (Closed { consecutive_failures }, Failure) => {
                let n = consecutive_failures + 1;
                if n >= self.failure_threshold {
                    Open { cooldown_remaining: self.cooldown_steps }
                } else {
                    Closed { consecutive_failures: n }
                }
            }
            (Closed { .. }, Idle) => *state,

            // Open: count down; only idle steps advance the timer since real
            // calls are rejected before they happen.
            (Open { cooldown_remaining: 0 }, _) => HalfOpen,
            (Open { cooldown_remaining }, _) => Open {
                cooldown_remaining: cooldown_remaining - 1,
            },

            // HalfOpen: probe call decides the next state.
            (HalfOpen, Success) => Closed { consecutive_failures: 0 },
            (HalfOpen, Failure) => Open { cooldown_remaining: self.cooldown_steps },
            (HalfOpen, Idle) => HalfOpen,
        };

        (next, (decision, next))
    }
}

fn main() {
    use CallOutcome::*;
    let mut breaker = Runner::new(CircuitBreaker::new(3, 4));

    // Scripted workload: three failures trip the breaker, calls are rejected
    // during the cooldown, a probe succeeds, and we're back to normal.
    let script = [
        Success, Success, Failure, // mild trouble
        Failure, Failure,           // trips: 3rd failure opens the circuit
        Failure, Failure,           // rejected while open
        Idle, Idle, Idle, Idle,     // cooldown ticks
        Success,                    // half-open probe succeeds → closed
        Success, Failure, Success,  // back to normal
    ];

    println!("step | outcome  | decision | state");
    let mut ever_rejected = false;
    let mut ever_halfopen = false;
    let mut ended_closed = false;

    for (t, &outcome) in script.iter().enumerate() {
        let (decision, state) = breaker.step(outcome);
        println!("{:4} | {:?} | {:?}  | {:?}", t, outcome, decision, state);
        if matches!(decision, Decision::Reject) {
            ever_rejected = true;
        }
        if matches!(state, BreakerState::HalfOpen) {
            ever_halfopen = true;
        }
        ended_closed = matches!(state, BreakerState::Closed { .. });
    }

    assert!(ever_rejected, "a rejection must happen during the open window");
    assert!(ever_halfopen, "the breaker must probe via HalfOpen");
    assert!(ended_closed, "successful probe + successes should close the breaker");
}
