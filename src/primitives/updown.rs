use crate::core::StateMachine;

/// Integer counter that increments on `Up` and decrements on `Down`. §4.1.1.2.
pub struct UpDown;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpDownInput {
    Up,
    Down,
}

impl StateMachine for UpDown {
    type Input = UpDownInput;
    type Output = i64;
    type State = i64;

    fn start_state(&self) -> i64 {
        0
    }

    fn next_values(&self, state: &i64, input: &UpDownInput) -> (i64, i64) {
        let next = match input {
            UpDownInput::Up => state + 1,
            UpDownInput::Down => state - 1,
        };
        (next, next)
    }
}
