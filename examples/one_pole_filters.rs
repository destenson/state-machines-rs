//! Two single-pole IIR filters — the minimum-viable DSP primitives, each
//! with exactly one unit of state and one multiply-add per sample.
//!
//! **EMA / one-pole low-pass**: `y[n] = α·x[n] + (1−α)·y[n−1]`.
//! A smoother. Passes slow changes, attenuates fast ones. The alpha knob
//! trades responsiveness against smoothing (α→1: pure passthrough; α→0:
//! frozen at the initial value). Used for sensor smoothing, metric
//! dashboards, envelope followers.
//!
//! **DC blocker / one-pole high-pass**: `y[n] = x[n] − x[n−1] + R·y[n−1]`.
//! Kills DC offset and very-low-frequency drift while leaving the
//! interesting signal alone. `R` close to 1 (typically 0.995–0.999) makes
//! the notch at DC tight. Used in audio (microphone bias removal),
//! biomedical (ECG baseline wander), sensor front-ends.
//!
//! Together they cover the complementary halves of the spectrum with
//! essentially free compute — one multiply and one add per sample.

use state_machines_rs::{Runner, StateMachine};

pub struct OnePoleLowPass {
    alpha: f64,
}

impl OnePoleLowPass {
    pub fn new(alpha: f64) -> Self {
        assert!((0.0..=1.0).contains(&alpha));
        Self { alpha }
    }
    /// EMA matched to the effective window length `n` samples.
    pub fn with_period(n: usize) -> Self {
        Self::new(2.0 / (n as f64 + 1.0))
    }
}

impl StateMachine for OnePoleLowPass {
    type Input = f64;
    type Output = f64;
    /// `(initialized, y)` — defer seeding until we see the first sample, so
    /// the output doesn't crawl out of zero.
    type State = (bool, f64);

    fn start_state(&self) -> (bool, f64) {
        (false, 0.0)
    }
    fn next_values(&self, state: &(bool, f64), x: &f64) -> ((bool, f64), f64) {
        let y = if state.0 { self.alpha * x + (1.0 - self.alpha) * state.1 } else { *x };
        ((true, y), y)
    }
}

/// High-pass via the canonical one-pole DC-blocker topology. Output is the
/// current sample minus a decaying average of all previous samples.
pub struct DcBlocker {
    r: f64,
}

impl DcBlocker {
    pub fn new(r: f64) -> Self {
        assert!((0.0..1.0).contains(&r));
        Self { r }
    }
}

impl StateMachine for DcBlocker {
    type Input = f64;
    type Output = f64;
    /// `(x_prev, y_prev)`.
    type State = (f64, f64);

    fn start_state(&self) -> (f64, f64) {
        (0.0, 0.0)
    }
    fn next_values(&self, state: &(f64, f64), x: &f64) -> ((f64, f64), f64) {
        let (x_prev, y_prev) = *state;
        let y = x - x_prev + self.r * y_prev;
        ((*x, y), y)
    }
}

fn mean(xs: &[f64]) -> f64 {
    xs.iter().sum::<f64>() / xs.len() as f64
}

fn rms(xs: &[f64]) -> f64 {
    let m = mean(xs);
    (xs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / xs.len() as f64).sqrt()
}

fn main() {
    // Test signal: strong DC offset + slow 5 Hz cosine + fast 200 Hz cosine,
    // sampled at 1 kHz.
    let fs = 1000.0;
    let n = 2000;
    let signal: Vec<f64> = (0..n)
        .map(|t| {
            let t = t as f64 / fs;
            5.0                                       // DC
                + (2.0 * std::f64::consts::PI * 5.0 * t).cos()   // slow
                + (2.0 * std::f64::consts::PI * 200.0 * t).cos() // fast
        })
        .collect();

    // EMA sized to ~20 samples smooths out the 200 Hz component (period 5
    // samples) but keeps the DC and the 5 Hz tone.
    let ema = Runner::new(OnePoleLowPass::with_period(20))
        .transduce(signal.iter().copied());

    // DC blocker with R=0.98 kills the offset and passes both tones.
    // (Higher R gives a tighter notch but also a longer startup transient —
    // 0.98 decays to ≈0 in a couple hundred samples.)
    let hp = Runner::new(DcBlocker::new(0.98)).transduce(signal.iter().copied());

    // Skip transients for measurement.
    let skip = 400;
    let raw_mean = mean(&signal[skip..]);
    let ema_mean = mean(&ema[skip..]);
    let hp_mean = mean(&hp[skip..]);
    let raw_rms = rms(&signal[skip..]);
    let ema_rms = rms(&ema[skip..]);
    let hp_rms = rms(&hp[skip..]);

    println!("input    mean = {:>7.4}   AC RMS = {:.4}", raw_mean, raw_rms);
    println!("EMA      mean = {:>7.4}   AC RMS = {:.4}", ema_mean, ema_rms);
    println!("DC-blk   mean = {:>7.4}   AC RMS = {:.4}", hp_mean, hp_rms);

    // EMA preserves the DC level. Its AC RMS is reduced (200 Hz is deeply
    // attenuated) but not dramatically — the 5 Hz tone sits well inside the
    // passband and comes through at full strength.
    assert!((ema_mean - 5.0).abs() < 0.05, "EMA must preserve DC offset");
    assert!(
        ema_rms < 0.8 * raw_rms,
        "EMA should attenuate the 200 Hz component, got {ema_rms} vs {raw_rms}"
    );

    // DC blocker flattens the mean (≥10× reduction from the 5.0 input offset)
    // while leaving most of the AC energy untouched.
    assert!(
        hp_mean.abs() < 0.5,
        "DC blocker should reduce mean by ≥10×, got {hp_mean}"
    );
    assert!(
        (hp_rms - raw_rms).abs() / raw_rms < 0.1,
        "DC blocker should leave AC content largely intact"
    );
}
