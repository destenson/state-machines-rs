//! Scalar Kalman filter.
//!
//! The Kalman filter is the optimal linear estimator for a noisy measurement
//! of a linear system. This one-dimensional version tracks a slowly-drifting
//! "true" value corrupted by Gaussian measurement noise, assuming a simple
//! random-walk process model.
//!
//! The machine's state is the pair `(x̂, P)`: the estimate of the hidden
//! variable and its variance. Each step:
//!
//! ```text
//! Predict:  P⁻ = P + Q                      (process noise inflates variance)
//! Gain:     K  = P⁻ / (P⁻ + R)              (how much to trust this sample)
//! Update:   x̂ = x̂ + K · (z − x̂)             (pull toward measurement)
//!           P  = (1 − K) · P⁻
//! ```
//!
//! `Q` governs responsiveness (higher = track fast changes); `R` governs
//! smoothing (higher = ignore noisy samples).

use state_machines_rs::{Runner, StateMachine};

pub struct Kalman1D {
    /// Process variance — expected step-to-step drift of the hidden value.
    q: f64,
    /// Measurement variance — noise magnitude of sensor readings.
    r: f64,
    /// Initial estimate and its variance.
    x0: f64,
    p0: f64,
}

impl StateMachine for Kalman1D {
    type Input = f64;
    type Output = f64;
    /// `(x_hat, P)`.
    type State = (f64, f64);

    fn start_state(&self) -> (f64, f64) {
        (self.x0, self.p0)
    }

    fn next_values(&self, state: &(f64, f64), z: &f64) -> ((f64, f64), f64) {
        let (x_hat, p) = *state;
        let p_pred = p + self.q;
        let k = p_pred / (p_pred + self.r);
        let x_new = x_hat + k * (z - x_hat);
        let p_new = (1.0 - k) * p_pred;
        ((x_new, p_new), x_new)
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
    let n = 500;
    let mut noise = lcg_noise(0x5AB0CAFE);

    // Canonical "estimate a constant from noisy measurements" setup — the
    // problem Kalman himself used as motivation. Truth is flat at 10.0;
    // measurements are corrupted with uniform noise (variance 1/3).
    let truth: Vec<f64> = vec![10.0; n];
    let measurements: Vec<f64> = truth.iter().map(|x| x + noise()).collect();

    // Low Q reflects our belief that the truth barely drifts; R matches the
    // measurement variance. With n samples the estimator variance shrinks
    // toward R/n, so MSE improvement over the raw sensor grows with time.
    let filter = Kalman1D { q: 1e-5, r: 0.33, x0: measurements[0], p0: 1.0 };
    let estimates = Runner::new(filter).transduce(measurements.iter().copied());

    let skip = 100;
    let mse = |a: &[f64], b: &[f64]| -> f64 {
        a.iter().zip(b).map(|(x, y)| (x - y).powi(2)).sum::<f64>() / a.len() as f64
    };
    let mse_raw = mse(&measurements[skip..], &truth[skip..]);
    let mse_filt = mse(&estimates[skip..], &truth[skip..]);

    println!("raw measurement MSE:  {:.5}", mse_raw);
    println!("kalman estimate MSE:  {:.5}", mse_filt);
    println!("MSE reduction factor: {:.1}×", mse_raw / mse_filt);

    // For a constant truth and modest n, Kalman should crush the raw MSE.
    assert!(
        mse_filt < 0.1 * mse_raw,
        "expected >10× MSE improvement, got raw={mse_raw} filt={mse_filt}"
    );
}
