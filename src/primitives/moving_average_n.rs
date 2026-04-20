//! Simple moving average over the last `n` inputs — the rolling-mean
//! counterpart of [`SumLastN`](super::SumLastN).
//!
//! During warm-up (fewer than `n` samples seen), the output is the
//! partial average `running_sum / filled` rather than a pre-filled-with-
//! zeros bias. That matches what most practitioners want for streaming
//! metrics; if you want the zero-padded semantics, cascade `SumLastN`
//! with `Gain(1/n)` directly.

use crate::core::StateMachine;

pub struct MovingAverageN {
    n: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MovingAverageNError {
    ZeroWindow,
}

impl core::fmt::Display for MovingAverageNError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ZeroWindow => write!(f, "MovingAverageN window size must be at least 1"),
        }
    }
}

impl std::error::Error for MovingAverageNError {}

impl MovingAverageN {
    pub fn new_with(n: usize) -> Result<Self, MovingAverageNError> {
        if n == 0 {
            return Err(MovingAverageNError::ZeroWindow);
        }
        Ok(Self { n })
    }

    pub fn window(&self) -> usize {
        self.n
    }
}

/// Ring-buffer state: `(buffer, write_idx, filled, running_sum)`. Same
/// shape as `SumLastN<f64>`; the division happens on output so that the
/// sum is maintained exactly and only rounded once per step.
impl StateMachine for MovingAverageN {
    type Input = f64;
    type Output = f64;
    type State = (Vec<f64>, usize, usize, f64);

    fn start_state(&self) -> Self::State {
        (vec![0.0; self.n], 0, 0, 0.0)
    }

    fn next_values(&self, state: &Self::State, input: &f64) -> (Self::State, f64) {
        let (buffer, write_idx, filled, running_sum) = state;
        let mut new_sum = running_sum + input;
        if *filled == self.n {
            new_sum -= buffer[*write_idx];
        }
        let mut buffer = buffer.clone();
        buffer[*write_idx] = *input;
        let new_idx = (write_idx + 1) % self.n;
        let new_filled = (*filled + 1).min(self.n);
        let avg = new_sum / new_filled as f64;
        ((buffer, new_idx, new_filled, new_sum), avg)
    }
}
