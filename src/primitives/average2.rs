use crate::core::StateMachine;

/// At time `t`, outputs the average of the current and previous input. State
/// stores the previous input. §4.1.1.5.
pub struct Average2 {
    start: f64,
}

impl Average2 {
    pub fn new(start: f64) -> Self {
        Self { start }
    }
}

impl Default for Average2 {
    fn default() -> Self {
        Self { start: 0.0 }
    }
}

impl StateMachine for Average2 {
    type Input = f64;
    type Output = f64;
    type State = f64;

    fn start_state(&self) -> f64 {
        self.start
    }

    fn next_values(&self, state: &f64, input: &f64) -> (f64, f64) {
        (*input, (state + input) / 2.0)
    }
}
