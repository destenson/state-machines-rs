use crate::core::StateMachine;

/// Boolean negation as a pure machine. Exercise 4.4: compose with `Feedback`
/// to get a machine that alternates true/false.
pub struct Negation;

impl StateMachine for Negation {
    type Input = Option<bool>;
    type Output = Option<bool>;
    type State = ();

    fn start_state(&self) {}

    fn next_values(&self, _: &(), input: &Option<bool>) -> ((), Option<bool>) {
        ((), input.map(|b| !b))
    }
}
