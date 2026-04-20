use crate::core::StateMachine;
use crate::defined::SafeAdd;

/// Output at time `t` is the sum of all inputs through time `t`. State is the
/// running total. §4.1.1.4.
pub struct Accumulator<T> {
    start: T,
}

impl<T: Clone> Accumulator<T> {
    pub fn new(start: T) -> Self {
        Self { start }
    }
}

impl<T: Default> Default for Accumulator<T> {
    fn default() -> Self {
        Self { start: T::default() }
    }
}

impl<T: SafeAdd + Clone> StateMachine for Accumulator<T> {
    type Input = T;
    type Output = T;
    type State = T;

    fn start_state(&self) -> T {
        self.start.clone()
    }

    fn next_values(&self, state: &T, input: &T) -> (T, T) {
        let next = state.safe_add(input);
        (next.clone(), next)
    }
}
