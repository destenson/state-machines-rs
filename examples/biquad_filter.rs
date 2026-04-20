//! Biquad IIR filter (Transposed Direct Form II).
//!
//! The workhorse of audio and control DSP. A single biquad section
//! implements:
//!
//! ```text
//! y[n] = b0*x[n] + z1[n-1]
//! z1[n] = b1*x[n] - a1*y[n] + z2[n-1]
//! z2[n] = b2*x[n] - a2*y[n]
//! ```
//!
//! The `z1`, `z2` state registers *are* the machine's state. Coefficients
//! here realize an RBJ cookbook low-pass at `f0 = fs/20` with Q=0.707 (a
//! two-pole Butterworth response).
//!
//! This example filters a sine wave buried in white noise and reports the
//! RMS of the filtered noise vs. the raw noise. Noise at frequencies above
//! the corner is attenuated by 40 dB/decade.

use state_machines_rs::{Runner, StateMachine};

/// RBJ low-pass coefficients — see <https://www.w3.org/TR/audio-eq-cookbook/>.
fn rbj_lowpass(sample_rate: f64, cutoff_hz: f64, q: f64) -> Biquad {
    let w0 = 2.0 * std::f64::consts::PI * cutoff_hz / sample_rate;
    let (sin_w, cos_w) = w0.sin_cos();
    let alpha = sin_w / (2.0 * q);

    let b0 = (1.0 - cos_w) / 2.0;
    let b1 = 1.0 - cos_w;
    let b2 = (1.0 - cos_w) / 2.0;
    let a0 = 1.0 + alpha;
    let a1 = -2.0 * cos_w;
    let a2 = 1.0 - alpha;

    // Normalize so a0 = 1.
    Biquad {
        b0: b0 / a0,
        b1: b1 / a0,
        b2: b2 / a0,
        a1: a1 / a0,
        a2: a2 / a0,
    }
}

pub struct Biquad {
    b0: f64,
    b1: f64,
    b2: f64,
    a1: f64,
    a2: f64,
}

impl StateMachine for Biquad {
    type Input = f64;
    type Output = f64;
    /// Two state registers `(z1, z2)` — the classical biquad internal state.
    type State = (f64, f64);

    fn start_state(&self) -> (f64, f64) {
        (0.0, 0.0)
    }

    fn next_values(&self, state: &(f64, f64), input: &f64) -> ((f64, f64), f64) {
        let (z1, z2) = *state;
        let x = *input;
        let y = self.b0 * x + z1;
        let new_z1 = self.b1 * x - self.a1 * y + z2;
        let new_z2 = self.b2 * x - self.a2 * y;
        ((new_z1, new_z2), y)
    }
}

/// Deterministic-ish pseudo-random white noise so the example is reproducible.
fn lcg_noise(seed: u64) -> impl FnMut() -> f64 {
    let mut s = seed;
    move || {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((s >> 32) as u32 as f64 / u32::MAX as f64) * 2.0 - 1.0
    }
}

fn rms(xs: &[f64]) -> f64 {
    let s: f64 = xs.iter().map(|x| x * x).sum();
    (s / xs.len() as f64).sqrt()
}

fn main() {
    let fs = 1000.0;
    let signal_hz = 10.0; // Well inside the passband.
    let cutoff_hz = 50.0;
    let n = 2000;

    let mut noise = lcg_noise(0xC0FFEE);
    let raw: Vec<f64> = (0..n)
        .map(|t| {
            let s = (2.0 * std::f64::consts::PI * signal_hz * t as f64 / fs).sin();
            s + 0.8 * noise()
        })
        .collect();

    let filter = rbj_lowpass(fs, cutoff_hz, std::f64::consts::FRAC_1_SQRT_2);
    let filtered = Runner::new(filter).transduce(raw.iter().copied());

    // Compare RMS of raw vs filtered, after skipping the transient.
    let skip = 200;
    let raw_rms = rms(&raw[skip..]);
    let filt_rms = rms(&filtered[skip..]);
    let attenuation_db = 20.0 * (filt_rms / raw_rms).log10();

    println!("raw RMS       = {:.4}", raw_rms);
    println!("filtered RMS  = {:.4}", filt_rms);
    println!("attenuation   = {:.2} dB", attenuation_db);

    // The pure signal has RMS ≈ 1/√2; the added noise dominates the raw RMS.
    // After filtering, most noise above the cutoff is gone, leaving something
    // close to the signal-only RMS.
    let signal_only_rms = std::f64::consts::FRAC_1_SQRT_2;
    assert!(filt_rms < raw_rms, "filter should reduce total energy");
    assert!(
        (filt_rms - signal_only_rms).abs() < 0.15,
        "filtered output should be close to the pure signal's RMS (~0.707), got {filt_rms}"
    );
}
