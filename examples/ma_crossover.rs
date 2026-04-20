//! Moving-average crossover — a textbook momentum trading signal expressed
//! as a state-machine pipeline.
//!
//! Topology: two exponential moving averages (fast + slow) run in parallel
//! on the same price stream. Their outputs feed a crossover detector that
//! compares them and emits Buy on an upward cross, Sell on a downward cross,
//! Hold otherwise. The detector's state is the sign of the previous
//! difference.
//!
//! ```text
//! price ─┬─► EMA(fast) ──┐
//!        │               ├─► CrossoverDetector ─► Buy/Sell/Hold
//!        └─► EMA(slow) ──┘
//! ```

use state_machines_rs::{Runner, SMExt, StateMachine};

/// Exponential moving average: `y[n] = α·x[n] + (1−α)·y[n−1]`.
pub struct Ema {
    alpha: f64,
}

impl Ema {
    pub fn new(alpha: f64) -> Self {
        assert!((0.0..=1.0).contains(&alpha));
        Self { alpha }
    }
    /// Build an EMA with the same effective window as an N-sample SMA.
    pub fn with_period(n: usize) -> Self {
        Self::new(2.0 / (n as f64 + 1.0))
    }
}

impl StateMachine for Ema {
    type Input = f64;
    type Output = f64;
    /// `(not_yet_initialized_flag, current_ema)`.
    type State = (bool, f64);

    fn start_state(&self) -> (bool, f64) {
        (true, 0.0)
    }
    fn next_values(&self, state: &(bool, f64), x: &f64) -> ((bool, f64), f64) {
        // Seed with the first sample so the EMA doesn't crawl out of zero.
        let next = if state.0 { *x } else { self.alpha * x + (1.0 - self.alpha) * state.1 };
        ((false, next), next)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Signal {
    Buy,
    Sell,
    Hold,
}

/// Watches the sign of `fast − slow`; emits Buy when it turns positive,
/// Sell when negative, Hold otherwise.
pub struct CrossoverDetector;

impl StateMachine for CrossoverDetector {
    type Input = (f64, f64);
    type Output = Signal;
    /// Sign of previous `fast − slow`: `-1`, `0` (uninitialized), `+1`.
    type State = i8;

    fn start_state(&self) -> i8 {
        0
    }
    fn next_values(&self, prev: &i8, (fast, slow): &(f64, f64)) -> (i8, Signal) {
        let diff = fast - slow;
        let sign = if diff > 0.0 { 1 } else if diff < 0.0 { -1 } else { 0 };
        let signal = match (*prev, sign) {
            (p, s) if p <= 0 && s > 0 => Signal::Buy,
            (p, s) if p >= 0 && s < 0 => Signal::Sell,
            _ => Signal::Hold,
        };
        (sign, signal)
    }
}

fn main() {
    // Synthetic price series: trending sine wave + gentle drift.
    let n = 500;
    let prices: Vec<f64> = (0..n)
        .map(|t| {
            let dt = t as f64;
            100.0 + 5.0 * (dt / 40.0).sin() + 0.03 * dt
        })
        .collect();

    let strategy = Ema::with_period(12)
        .parallel(Ema::with_period(48))
        .cascade(CrossoverDetector);

    let signals = Runner::new(strategy).transduce(prices.iter().copied());

    let buys = signals.iter().filter(|s| **s == Signal::Buy).count();
    let sells = signals.iter().filter(|s| **s == Signal::Sell).count();

    println!("generated {} buy and {} sell signals over {} samples", buys, sells, n);
    for (t, s) in signals.iter().enumerate().filter(|(_, s)| **s != Signal::Hold) {
        println!("  t={:3}  price={:.3}  signal={:?}", t, prices[t], s);
    }

    // Trend + oscillation must produce *some* crossings but not chatter.
    assert!(buys >= 2 && sells >= 2, "expect repeated momentum reversals");
    assert!(buys + sells < n / 4, "signals should not chatter every few bars");
}
