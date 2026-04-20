use crate::core::StateMachine;
use crate::defined::SafeAdd;

/// At time `t`, outputs `i[t-2] + i[t-1] + i[t]`. Demonstrates a machine whose
/// state is a fixed-size tuple of past inputs. §4.1.2.3.
pub struct SumLast3<T> {
    start: (T, T),
}

impl<T: Clone> SumLast3<T> {
    pub fn new(start: (T, T)) -> Self {
        Self { start }
    }
}

impl<T: Default + Clone> Default for SumLast3<T> {
    fn default() -> Self {
        Self { start: (T::default(), T::default()) }
    }
}

impl<T: SafeAdd + Clone> StateMachine for SumLast3<T> {
    type Input = T;
    type Output = T;
    type State = (T, T);

    fn start_state(&self) -> (T, T) {
        self.start.clone()
    }

    fn next_values(&self, state: &(T, T), input: &T) -> ((T, T), T) {
        let (prev_prev, prev) = state;
        let sum = prev_prev.safe_add(&prev.safe_add(input));
        ((prev.clone(), input.clone()), sum)
    }
}
