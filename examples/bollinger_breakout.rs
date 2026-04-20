//! Bollinger band breakout detector — trade when price departs from a rolling
//! mean by more than K standard deviations.
//!
//! The machine keeps a fixed-length ring buffer of the last `period` samples.
//! Each step it pushes the new price, computes mean and standard deviation
//! across the window, and emits one of:
//!
//! * `BreakoutUp`   — price pierced the upper band
//! * `BreakoutDown` — price pierced the lower band
//! * `Hold`         — inside the band, or window not yet full
//!
//! Unlike the EMA-based crossover example, this pattern explicitly uses
//! *volatility* (the rolling σ) rather than trend slope to size the trigger,
//! so it adapts to quiet vs. noisy regimes.

use state_machines_rs::{Runner, StateMachine};

const PERIOD: usize = 20;
const K_STD: f64 = 2.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Breakout {
    Up,
    Down,
    Hold,
}

pub struct Bollinger {
    period: usize,
    k: f64,
}

impl Bollinger {
    pub fn new(period: usize, k: f64) -> Self {
        assert!(period >= 2);
        Self { period, k }
    }
}

impl StateMachine for Bollinger {
    type Input = f64;
    type Output = Breakout;
    /// `window` is a fixed-capacity ring buffer; `filled` tracks how many
    /// samples we've seen so far (so we don't emit signals before the window
    /// is full). `write_idx` is the next slot to overwrite.
    type State = (Vec<f64>, usize, usize);

    fn start_state(&self) -> Self::State {
        (vec![0.0; self.period], 0, 0)
    }

    fn next_values(&self, state: &Self::State, price: &f64) -> (Self::State, Breakout) {
        let (window, filled, write_idx) = state;
        let mut window = window.clone();
        window[*write_idx] = *price;
        let new_idx = (write_idx + 1) % self.period;
        let new_filled = (*filled + 1).min(self.period);

        let signal = if new_filled < self.period {
            Breakout::Hold
        } else {
            let mean = window.iter().sum::<f64>() / self.period as f64;
            let var = window.iter().map(|x| (x - mean).powi(2)).sum::<f64>()
                / self.period as f64;
            let std = var.sqrt();
            if *price > mean + self.k * std {
                Breakout::Up
            } else if *price < mean - self.k * std {
                Breakout::Down
            } else {
                Breakout::Hold
            }
        };

        ((window, new_filled, new_idx), signal)
    }
}

fn lcg_noise(seed: u64) -> impl FnMut() -> f64 {
    let mut s = seed;
    move || {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((s >> 32) as u32 as f64 / u32::MAX as f64) * 2.0 - 1.0
    }
}

fn main() {
    let n = 400;
    let mut noise = lcg_noise(0xB011);

    // Synthetic price: stationary process around 100 (what Bollinger bands
    // are designed for), then an injected "shock" around t=250, then back to
    // quiet. The filter should fire during the shock and stay quiet otherwise.
    let prices: Vec<f64> = (0..n)
        .map(|t| {
            let base = 100.0;
            let shock = if (240..=260).contains(&t) {
                5.0 * (((t - 240) as f64) * 0.5).sin()
            } else {
                0.0
            };
            base + 0.3 * noise() + shock
        })
        .collect();

    let signals = Runner::new(Bollinger::new(PERIOD, K_STD)).transduce(prices.iter().copied());

    let fired_inside_shock = signals[240..=260]
        .iter()
        .filter(|s| **s != Breakout::Hold)
        .count();
    let fired_outside = signals
        .iter()
        .enumerate()
        .filter(|(t, s)| !(240..=260).contains(t) && **s != Breakout::Hold)
        .count();

    println!("signals during shock (t=240..=260): {}", fired_inside_shock);
    println!("signals outside shock:              {}", fired_outside);
    for (t, s) in signals.iter().enumerate().filter(|(_, s)| **s != Breakout::Hold) {
        println!("  t={:3}  price={:.3}  signal={:?}", t, prices[t], s);
    }

    // The shock must trigger something; the quiet regime must stay quiet.
    assert!(fired_inside_shock >= 1, "shock should produce a breakout");
    assert!(
        fired_outside <= 5,
        "quiet regime shouldn't fire often; got {fired_outside}"
    );
}
