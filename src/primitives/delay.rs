use crate::core::StateMachine;

/// Passes its input through with one step of delay; the chapter's `R` operator.
/// At time `t`, output is input from time `t-1` (with an initial value at t=0).
/// §4.1.1.3.
pub struct Delay<T> {
    init: T,
}

impl<T: Clone> Delay<T> {
    pub fn new(init: T) -> Self {
        Self { init }
    }
}

impl<T: Clone> StateMachine for Delay<T> {
    type Input = T;
    type Output = T;
    type State = T;

    fn start_state(&self) -> T {
        self.init.clone()
    }

    fn next_values(&self, state: &T, input: &T) -> (T, T) {
        (input.clone(), state.clone())
    }
}
