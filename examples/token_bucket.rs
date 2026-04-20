//! Token-bucket rate limiter.
//!
//! A bucket holds up to `capacity` tokens and refills at `rate` tokens per
//! step. A request for `k` tokens is *allowed* if the bucket currently holds
//! at least `k` (bucket is drained by `k`), and *denied* otherwise (bucket
//! is left alone).
//!
//! Used everywhere real systems meet real users: HTTP APIs (GitHub, Stripe,
//! AWS), trading gateways, DNS servers, SSH logins, network QoS. The
//! cheap-burstable-but-bounded shape — bursts of up to `capacity`, sustained
//! throughput capped at `rate` — is exactly what most throttling problems
//! want.
//!
//! State is a single `f64`: the current token count (fractional so the
//! refill rate can be non-integer).

use state_machines_rs::{Runner, StateMachine};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Allowed,
    Denied,
}

pub struct TokenBucket {
    capacity: f64,
    refill_per_step: f64,
}

impl TokenBucket {
    pub fn new(capacity: f64, refill_per_step: f64) -> Self {
        assert!(capacity > 0.0 && refill_per_step >= 0.0);
        Self { capacity, refill_per_step }
    }
}

impl StateMachine for TokenBucket {
    /// Number of tokens requested this step. `0` models an idle tick (still
    /// lets the bucket refill).
    type Input = u32;
    type Output = Decision;
    /// Current token count (saturated at `capacity`).
    type State = f64;

    fn start_state(&self) -> f64 {
        self.capacity
    }

    fn next_values(&self, tokens: &f64, want: &u32) -> (f64, Decision) {
        // Refill first (classic token-bucket: the bucket accrues regardless
        // of whether a request arrives).
        let refilled = (tokens + self.refill_per_step).min(self.capacity);
        let want_f = *want as f64;
        if refilled >= want_f {
            (refilled - want_f, Decision::Allowed)
        } else {
            (refilled, Decision::Denied)
        }
    }
}

fn main() {
    // 5-token capacity, refilling at 0.1 tokens/step. A steady 1-per-step
    // demand will therefore outrun the bucket and stall after the initial
    // burst is exhausted.
    let bucket = TokenBucket::new(5.0, 0.1);
    let mut r = Runner::new(bucket);

    // Scripted workload. `0` = idle tick (bucket refills, nothing asked).
    let burst_len = 8;
    let idle_short = 20;
    let idle_long = 30;
    let mut script: Vec<u32> = Vec::new();
    script.extend(std::iter::repeat_n(1u32, burst_len));   // burst → 5 allow, 3 deny
    script.extend(std::iter::repeat_n(0u32, idle_short));  // refill ~2 tokens
    script.extend([1, 1]);                                 // small requests: allow
    script.push(3);                                        // big request: deny
    script.extend(std::iter::repeat_n(0u32, idle_long));   // refill to ≥ 3
    script.push(3);                                        // big request: allow

    let mut decisions = Vec::with_capacity(script.len());
    let mut allowed = 0;
    let mut denied = 0;
    println!("step | want | decision | remaining tokens");
    for (t, &want) in script.iter().enumerate() {
        let dec = r.step(want);
        decisions.push(dec);
        println!("{:4} | {:>3}  | {:?}  | {:.2}", t, want, dec, r.state());
        match dec {
            Decision::Allowed => allowed += 1,
            Decision::Denied => denied += 1,
        }
    }
    println!("\nallowed = {}, denied = {}", allowed, denied);

    // Burst: 5 allowed (drains capacity 5), 3 denied.
    let burst_allowed = decisions[..burst_len]
        .iter()
        .filter(|d| **d == Decision::Allowed)
        .count();
    assert_eq!(burst_allowed, 5, "initial burst must drain the bucket and no more");

    // Small requests after short idle should pass.
    let small_start = burst_len + idle_short;
    assert!(decisions[small_start..small_start + 2]
        .iter()
        .all(|d| *d == Decision::Allowed));

    // The 3-token request right after the small requests must be denied
    // (bucket drained below 3).
    assert_eq!(decisions[small_start + 2], Decision::Denied);

    // After the long idle, the final 3-token request must succeed.
    assert_eq!(*decisions.last().unwrap(), Decision::Allowed);
}
