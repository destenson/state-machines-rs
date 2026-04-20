use crate::core::StateMachine;
use crate::defined::{Defined, SafeMul};

/// Takes a pair `(a, b)` and emits `a * b`. Undefined-propagating analog of
/// [`Adder`](super::Adder); used in the factorial example (§4.2.3.5).
pub struct Multiplier<T> {
    _phantom: core::marker::PhantomData<T>,
}

impl<T> Multiplier<T> {
    pub fn new() -> Self {
        Self { _phantom: core::marker::PhantomData }
    }
}

impl<T> Default for Multiplier<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> StateMachine for Multiplier<T>
where
    T: SafeMul + Defined,
{
    type Input = (T, T);
    type Output = T;
    type State = T;

    fn start_state(&self) -> T {
        T::undefined()
    }

    fn next_values(&self, _: &T, input: &(T, T)) -> (T, T) {
        let out = input.0.safe_mul(&input.1);
        (out.clone(), out)
    }
}
