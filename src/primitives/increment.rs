use crate::core::StateMachine;
use crate::defined::{Defined, SafeAdd};

/// Pure adder of a constant: output is `input + incr`. Used as the canonical
/// "has no direct output dependence... wait, yes it does" example; the chapter
/// cascades it with [`Delay`](super::Delay) to build a counter under feedback.
/// §4.2.1.
pub struct Increment<T> {
    incr: T,
}

impl<T: Clone> Increment<T> {
    pub fn new(incr: T) -> Self {
        Self { incr }
    }
}

impl<T> StateMachine for Increment<T>
where
    T: SafeAdd + Defined,
{
    type Input = T;
    type Output = T;
    type State = T;

    fn start_state(&self) -> T {
        T::undefined()
    }

    fn next_values(&self, _: &T, input: &T) -> (T, T) {
        let out = input.safe_add(&self.incr);
        (out.clone(), out)
    }
}
