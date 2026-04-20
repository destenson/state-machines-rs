use crate::core::StateMachine;
use crate::defined::{Defined, SafeAdd};

/// Takes a pair `(a, b)` as input and emits `a + b`. Pure; undefined on either
/// input produces undefined output (via [`SafeAdd`]).
///
/// The chapter's Fibonacci example uses this to sum two delayed streams.
pub struct Adder<T> {
    _phantom: core::marker::PhantomData<T>,
}

impl<T> Adder<T> {
    pub fn new() -> Self {
        Self { _phantom: core::marker::PhantomData }
    }
}

impl<T> Default for Adder<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> StateMachine for Adder<T>
where
    T: SafeAdd + Defined,
{
    type Input = (T, T);
    type Output = T;
    type State = T;

    fn start_state(&self) -> T {
        T::undefined()
    }

    fn next_values(&self, _: &T, input: &(T, T)) -> (T, T) {
        let out = input.0.safe_add(&input.1);
        (out.clone(), out)
    }
}
