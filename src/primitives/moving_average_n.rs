//! Simple moving average over the last `n` inputs — the rolling-mean
//! counterpart of [`SumLastN`](super::SumLastN).
//!
//! During warm-up (fewer than `n` samples seen), the output is the
//! partial average `running_sum / filled` rather than a pre-filled-with-
//! zeros bias. That matches what most practitioners want for streaming
//! metrics; if you want the zero-padded semantics, cascade `SumLastN`
//! with `Gain(1/n)` directly.

use crate::core::StateMachine;
use crate::primitives::ring_buffer::RingBuffer;

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

impl StateMachine for MovingAverageN {
    type Input = f64;
    type Output = f64;
    /// `(ring_buffer, running_sum)`. Running sum is accumulated exactly;
    /// division happens only on output so the stored state isn't rounded
    /// every step.
    type State = (RingBuffer<f64>, f64);

    fn start_state(&self) -> Self::State {
        (RingBuffer::new(self.n), 0.0)
    }

    fn next_values(&self, state: &Self::State, input: &f64) -> (Self::State, f64) {
        let (buffer, sum) = state;
        let mut buffer = buffer.clone();
        let evicted = buffer.push(*input);
        let mut new_sum = sum + input;
        if let Some(old) = evicted {
            new_sum -= old;
        }
        let avg = new_sum / buffer.filled() as f64;
        ((buffer, new_sum), avg)
    }
}
