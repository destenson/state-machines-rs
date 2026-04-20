//! Rolling-window sum over the last `n` inputs.
//!
//! Each step the output is `input[t] + input[t-1] + ... + input[t-n+1]`,
//! or the running partial sum during the warm-up phase before the window
//! is full. Updates are O(1): we keep a running sum in state and, once
//! the buffer fills, subtract the evicted value from the sum.
//!
//! Generic over any numeric type implementing [`SafeAdd`] + [`SafeSub`]
//! (integers, floats, `Option<T>` for feedback pipelines).
//!
//! [`SafeAdd`]: crate::SafeAdd
//! [`SafeSub`]: crate::SafeSub

use crate::core::StateMachine;
use crate::defined::{SafeAdd, SafeSub};
use crate::primitives::ring_buffer::RingBuffer;

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

impl<T> StateMachine for SumLastN<T>
where
    T: SafeAdd + SafeSub + Clone + Default,
{
    type Input = T;
    type Output = T;
    /// `(ring_buffer, running_sum)`.
    type State = (RingBuffer<T>, T);

    fn start_state(&self) -> Self::State {
        (RingBuffer::new(self.n), T::default())
    }

    fn next_values(&self, state: &Self::State, input: &T) -> (Self::State, T) {
        let (buffer, sum) = state;
        let mut buffer = buffer.clone();
        let evicted = buffer.push(input.clone());
        let mut new_sum = sum.safe_add(input);
        if let Some(old) = evicted {
            new_sum = new_sum.safe_sub(&old);
        }
        ((buffer, new_sum.clone()), new_sum)
    }
}
