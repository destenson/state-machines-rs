use crate::core::StateMachine;
use crate::defined::SafeMul;

/// Pure multiplier: output is `k * input`. Stateless (state type is `()`).
/// §4.1.2.2.1.
pub struct Gain<T> {
    k: T,
}

impl<T: Clone> Gain<T> {
    pub fn new(k: T) -> Self {
        Self { k }
    }
}

impl<T: SafeMul + Clone> StateMachine for Gain<T> {
    type Input = T;
    type Output = T;
    type State = ();

    fn start_state(&self) {}

    fn next_values(&self, _: &(), input: &T) -> ((), T) {
        ((), self.k.safe_mul(input))
    }
}
