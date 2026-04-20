//! PID controller built compositionally from library primitives.
//!
//! The controller is the canonical sum `Kp·e + Ki·∫e + Kd·de/dt`, which
//! maps onto our combinators as:
//!
//! ```text
//! P-branch:  Gain(Kp)
//! I-branch:  Accumulator · Gain(Ki)
//! D-branch:  (Wire + (-1)·Delay) · Gain(Kd)
//! PID:       P-branch ⊕ I-branch ⊕ D-branch      (via ParallelAdd)
//! ```
//!
//! The closed loop wires this into a first-order plant under `Feedback`:
//!
//! ```text
//! measurement ─► (setpoint − measurement) ─► PID ─► plant ─► measurement …
//! ```
//!
//! We drive a plant with `τ = 2.0 s` to a setpoint of 1.0 and watch it
//! converge.

use state_machines_rs::{
    Runner, SMExt, StateMachine,
    primitives::{Accumulator, Delay, Gain, Wire},
};

/// Computes `setpoint − measurement`. One-line pure SM.
struct SetpointError {
    setpoint: f64,
}

impl StateMachine for SetpointError {
    type Input = Option<f64>;
    type Output = Option<f64>;
    type State = ();
    fn start_state(&self) {}
    fn next_values(&self, _: &(), m: &Option<f64>) -> ((), Option<f64>) {
        ((), m.map(|m| self.setpoint - m))
    }
}

/// First-order lag: `τ·dy/dt + y = K·u`, discretized with forward Euler.
/// Output reflects the *previous* state, so the plant has no direct
/// input-to-output dependence — a prerequisite for closing the feedback loop.
struct FirstOrderPlant {
    dt: f64,
    tau: f64,
    gain: f64,
}

impl StateMachine for FirstOrderPlant {
    type Input = Option<f64>;
    type Output = Option<f64>;
    type State = Option<f64>;

    fn start_state(&self) -> Option<f64> {
        Some(0.0)
    }
    fn next_values(
        &self,
        state: &Option<f64>,
        u: &Option<f64>,
    ) -> (Option<f64>, Option<f64>) {
        let next = match (state, u) {
            (Some(y), Some(u)) => Some(y + self.dt / self.tau * (self.gain * u - y)),
            (Some(y), None) => Some(*y),
            _ => None,
        };
        (next, *state)
    }
}

fn main() {
    let kp = Some(0.8);
    let ki = Some(0.1);
    let kd = Some(0.2);

    let p_branch = Gain::new(kp);
    let i_branch = Accumulator::<Option<f64>>::new(Some(0.0)).cascade(Gain::new(ki));
    // (x − x[t−1]) as `Wire ⊕ Delay·Gain(−1)`.
    let d_branch = Wire::<Option<f64>>::new()
        .parallel_add(Delay::new(Some(0.0)).cascade(Gain::new(Some(-1.0))))
        .cascade(Gain::new(kd));

    let pid = p_branch.parallel_add(i_branch).parallel_add(d_branch);

    let setpoint = 1.0;
    let plant = FirstOrderPlant { dt: 0.1, tau: 2.0, gain: 1.0 };

    let system = SetpointError { setpoint }
        .cascade(pid)
        .cascade(plant)
        .feedback();

    let out: Vec<f64> = Runner::new(system)
        .run(200)
        .into_iter()
        .flatten()
        .collect();

    println!("t | measurement");
    for (t, y) in out.iter().enumerate().step_by(10) {
        println!("{:3} | {:+.6}", t, y);
    }

    let steady_state = out.last().copied().unwrap();
    println!("\nsteady-state error = {:+.6}", setpoint - steady_state);

    // Monotonic-ish approach to the setpoint; final error well under 1%.
    assert!((setpoint - steady_state).abs() < 0.02);
    // No wild overshoot: peak must stay below 1.5× the setpoint.
    let peak = out.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    assert!(peak < 1.5 * setpoint, "overshoot too large: peak = {peak}");
}
