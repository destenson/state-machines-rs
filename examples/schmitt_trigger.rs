//! Schmitt trigger — threshold-with-hysteresis, the classic cure for chattering
//! decisions on a noisy signal.
//!
//! A naive comparator `x > threshold` on a noisy input fires many times
//! around the threshold. A Schmitt trigger uses two thresholds: it only
//! flips high when the input crosses the *upper* bound, and only flips low
//! when it crosses the *lower* bound. Between the two, the output is held
//! steady. That memory is the state.
//!
//! Use cases: door sensors, optical encoders, relay drivers, TTL input
//! conditioning, any digital decision made from a noisy analog source.

use state_machines_rs::{Runner, StateMachine};

pub struct Schmitt {
    lower: f64,
    upper: f64,
}

impl Schmitt {
    pub fn new(lower: f64, upper: f64) -> Self {
        assert!(lower < upper);
        Self { lower, upper }
    }
}

impl StateMachine for Schmitt {
    type Input = f64;
    type Output = bool;
    type State = bool;

    fn start_state(&self) -> bool {
        false
    }

    fn next_values(&self, high: &bool, x: &f64) -> (bool, bool) {
        let next = if *high {
            *x >= self.lower // stay high until we drop below the lower bound
        } else {
            *x >= self.upper // stay low until we rise above the upper bound
        };
        (next, next)
    }
}

fn lcg_noise(seed: u64) -> impl FnMut() -> f64 {
    let mut s = seed;
    move || {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((s >> 32) as u32 as f64 / u32::MAX as f64) * 2.0 - 1.0
    }
}

/// Count rising edges in a bool stream.
fn edges(xs: &[bool]) -> usize {
    xs.windows(2).filter(|w| !w[0] && w[1]).count()
}

fn main() {
    let n = 2000;
    let mut noise = lcg_noise(0xBADC0DE);

    // Square wave with levels 0.8 / 0.2 and moderate noise (amplitude 0.4).
    // The signal alone crosses the Schmitt bands (0.3 / 0.7) but the noise
    // alone does not, so the Schmitt output tracks the underlying square
    // wave exactly. Meanwhile the noise freely crosses the naive comparator
    // threshold at 0.5 during every phase, producing chatter.
    let raw: Vec<f64> = (0..n)
        .map(|t| if (t / 100) % 2 == 0 { 0.8 } else { 0.2 } + 0.4 * noise())
        .collect();

    // Naive comparator at 0.5 — chatters whenever noise crosses.
    let naive: Vec<bool> = raw.iter().map(|x| *x > 0.5).collect();

    // Schmitt with 0.3 / 0.7 thresholds — ignores noise inside the band.
    let schmitt = Runner::new(Schmitt::new(0.3, 0.7)).transduce(raw.iter().copied());

    let true_edges = n / 200;
    let naive_edges = edges(&naive);
    let schmitt_edges = edges(&schmitt);

    println!("true rising edges in stimulus: {}", true_edges);
    println!("naive comparator edges:        {}   (includes noise chatter)", naive_edges);
    println!("schmitt trigger edges:         {}", schmitt_edges);

    // Schmitt must produce exactly the rising edges in the underlying square
    // wave, while the naive comparator must chatter by at least an order of
    // magnitude.
    assert_eq!(schmitt_edges, true_edges);
    assert!(naive_edges > 5 * schmitt_edges);
}
