//! Rolling-window variance over the last `n` samples (f64).
//!
//! Maintains running sums of `x` and `x²` in state, updating in O(1) per
//! step once the window saturates. Variance is computed from these two
//! moments via the identity
//!
//! ```text
//! Σ(x_i - μ)² = Σx_i² - (Σx_i)² / n
//! ```
//!
//! which is fast but sensitive to catastrophic cancellation when the
//! values are large and the variance is small. For typical streaming
//! metrics and DSP use this is fine; applications needing maximum
//! precision over extreme input ranges should use a Welford-style
//! updater instead.
//!
//! Two variants are supported at construction time:
//!
//! - **Population** (`new_population_with`): divide by `n` (or by the
//!   number of samples seen so far during warm-up). Emits `0.0` for the
//!   first sample since a single point has no spread.
//! - **Sample** (`new_sample_with`): divide by `n - 1` (Bessel's
//!   correction), the unbiased estimator. Emits `NaN` until at least
//!   two samples have arrived.

use crate::core::StateMachine;
use crate::primitives::ring_buffer::RingBuffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarianceKind {
    Population,
    Sample,
}

pub struct VarianceLastN {
    n: usize,
    kind: VarianceKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VarianceLastNError {
    ZeroWindow,
    /// Sample variance needs at least two slots so the `n - 1` divisor is
    /// positive even when the window is full.
    SampleWindowTooSmall,
}

impl core::fmt::Display for VarianceLastNError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ZeroWindow => write!(f, "VarianceLastN window size must be at least 1"),
            Self::SampleWindowTooSmall => {
                write!(f, "VarianceLastN sample variance requires window size ≥ 2")
            }
        }
    }
}

impl std::error::Error for VarianceLastNError {}

impl VarianceLastN {
    pub fn new_population_with(n: usize) -> Result<Self, VarianceLastNError> {
        if n == 0 {
            return Err(VarianceLastNError::ZeroWindow);
        }
        Ok(Self { n, kind: VarianceKind::Population })
    }

    pub fn new_sample_with(n: usize) -> Result<Self, VarianceLastNError> {
        if n < 2 {
            return Err(VarianceLastNError::SampleWindowTooSmall);
        }
        Ok(Self { n, kind: VarianceKind::Sample })
    }

    pub fn window(&self) -> usize {
        self.n
    }

    pub fn kind(&self) -> VarianceKind {
        self.kind
    }
}

impl StateMachine for VarianceLastN {
    type Input = f64;
    type Output = f64;
    /// `(ring_buffer, sum, sum_of_squares)`.
    type State = (RingBuffer<f64>, f64, f64);

    fn start_state(&self) -> Self::State {
        (RingBuffer::new(self.n), 0.0, 0.0)
    }

    fn next_values(&self, state: &Self::State, input: &f64) -> (Self::State, f64) {
        let (buffer, sum, sum_sq) = state;
        let x = *input;
        let mut buffer = buffer.clone();
        let evicted = buffer.push(x);
        let mut new_sum = sum + x;
        let mut new_sum_sq = sum_sq + x * x;
        if let Some(old) = evicted {
            new_sum -= old;
            new_sum_sq -= old * old;
        }
        let filled = buffer.filled() as f64;
        // Sum of squared deviations via the two-moment identity. Floor at
        // zero to absorb floating-point roundoff near constant inputs.
        let ssd = (new_sum_sq - new_sum * new_sum / filled).max(0.0);
        let variance = match self.kind {
            VarianceKind::Population => ssd / filled,
            VarianceKind::Sample => {
                if buffer.filled() < 2 {
                    f64::NAN
                } else {
                    ssd / (filled - 1.0)
                }
            }
        };
        ((buffer, new_sum, new_sum_sq), variance)
    }
}
