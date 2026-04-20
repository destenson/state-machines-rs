use crate::core::StateMachine;

/// Trivial TSM that emits its fixed character once and then terminates.
/// Ignores input. §4.3.1.
pub struct CharTSM {
    c: char,
}

impl CharTSM {
    pub fn new(c: char) -> Self {
        Self { c }
    }
}

impl StateMachine for CharTSM {
    type Input = ();
    type Output = char;
    /// `false` = not yet done, `true` = emitted.
    type State = bool;

    fn start_state(&self) -> bool {
        false
    }

    fn next_values(&self, _: &bool, _: &()) -> (bool, char) {
        (true, self.c)
    }

    fn done(&self, state: &bool) -> bool {
        *state
    }
}
