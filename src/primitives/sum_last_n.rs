//! Rolling-window sum over the last `n` inputs.
//!
//! Each step the output is `input[t] + input[t-1] + ... + input[t-n+1]`,
//! or the running partial sum during the warm-up phase before the window
//! is full. Updates are O(1): we keep a running sum in state and, once
//! the window is full, subtract the value leaving the window and add the
//! new one.
//!
//! Generic over any numeric type implementing [`SafeAdd`] + [`SafeSub`]
//! (integers, floats, `Option<T>` for feedback pipelines).
//!
//! [`SafeAdd`]: crate::SafeAdd
//! [`SafeSub`]: crate::SafeSub

use crate::core::StateMachine;
use crate::defined::{SafeAdd, SafeSub};

pub struct SumLastN<T> {
    n: usize,
    _phantom: core::marker::PhantomData<fn(T) -> T>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SumLastNError {
    ZeroWindow,
}

impl core::fmt::Display for SumLastNError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ZeroWindow => write!(f, "SumLastN window size must be at least 1"),
        }
    }
}

impl std::error::Error for SumLastNError {}

impl<T> SumLastN<T> {
    pub fn new_with(n: usize) -> Result<Self, SumLastNError> {
        if n == 0 {
            return Err(SumLastNError::ZeroWindow);
        }
        Ok(Self { n, _phantom: core::marker::PhantomData })
    }

    pub fn window(&self) -> usize {
        self.n
    }
}

/// Ring-buffer state: `(buffer, write_idx, filled, running_sum)`.
///
/// - `buffer` has length `n`, with the oldest sample at `write_idx` once
///   the window fills up.
/// - `filled` counts how many real samples we've seen, capped at `n`. The
///   subtract step only kicks in once the buffer is saturated.
impl<T> StateMachine for SumLastN<T>
where
    T: SafeAdd + SafeSub + Clone + Default,
{
    type Input = T;
    type Output = T;
    type State = (Vec<T>, usize, usize, T);

    fn start_state(&self) -> Self::State {
        (vec![T::default(); self.n], 0, 0, T::default())
    }

    fn next_values(&self, state: &Self::State, input: &T) -> (Self::State, T) {
        let (buffer, write_idx, filled, running_sum) = state;
        let mut new_sum = running_sum.safe_add(input);
        if *filled == self.n {
            new_sum = new_sum.safe_sub(&buffer[*write_idx]);
        }
        let mut buffer = buffer.clone();
        buffer[*write_idx] = input.clone();
        let new_idx = (write_idx + 1) % self.n;
        let new_filled = (*filled + 1).min(self.n);
        (
            (buffer, new_idx, new_filled, new_sum.clone()),
            new_sum,
        )
    }
}
