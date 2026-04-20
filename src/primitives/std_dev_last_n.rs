//! Rolling standard deviation — literally
//! `VarianceLastN`'s output with `sqrt` applied.
//!
//! Implemented as a wrapper around [`VarianceLastN`](super::VarianceLastN):
//! same state, same update, output is `variance.sqrt()`. This is the
//! library's canonical small example of composing a primitive out of
//! another primitive rather than copy-pasting its implementation.

use crate::core::StateMachine;
use crate::primitives::variance_last_n::{VarianceKind, VarianceLastN, VarianceLastNError};

pub struct StdDevLastN {
    inner: VarianceLastN,
}

impl StdDevLastN {
    pub fn new_population_with(n: usize) -> Result<Self, VarianceLastNError> {
        Ok(Self { inner: VarianceLastN::new_population_with(n)? })
    }

    pub fn new_sample_with(n: usize) -> Result<Self, VarianceLastNError> {
        Ok(Self { inner: VarianceLastN::new_sample_with(n)? })
    }

    pub fn window(&self) -> usize {
        self.inner.window()
    }

    pub fn kind(&self) -> VarianceKind {
        self.inner.kind()
    }
}

impl StateMachine for StdDevLastN {
    type Input = f64;
    type Output = f64;
    type State = <VarianceLastN as StateMachine>::State;

    fn start_state(&self) -> Self::State {
        self.inner.start_state()
    }

    fn next_values(&self, state: &Self::State, input: &f64) -> (Self::State, f64) {
        let (new_state, variance) = self.inner.next_values(state, input);
        (new_state, variance.sqrt())
    }
}
