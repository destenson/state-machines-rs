use crate::core::StateMachine;

/// Language acceptor from §4.1.1.1: outputs `true` as long as the input stream
/// is a prefix of the infinite pattern `a, b, c, a, b, c, ...`. On any
/// deviation the machine enters the rejecting state `Reject` and outputs
/// `false` forever.
pub struct ABC;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AbcState {
    ExpectA,
    ExpectB,
    ExpectC,
    Reject,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AbcOutput {
    Accept,
    Reject,
}

impl AbcOutput {
    pub fn is_accept(self) -> bool {
        matches!(self, AbcOutput::Accept)
    }
}

impl StateMachine for ABC {
    type Input = char;
    type Output = AbcOutput;
    type State = AbcState;

    fn start_state(&self) -> AbcState {
        AbcState::ExpectA
    }

    fn next_values(&self, state: &AbcState, input: &char) -> (AbcState, AbcOutput) {
        let next = match (state, input) {
            (AbcState::ExpectA, 'a') => AbcState::ExpectB,
            (AbcState::ExpectB, 'b') => AbcState::ExpectC,
            (AbcState::ExpectC, 'c') => AbcState::ExpectA,
            _ => AbcState::Reject,
        };
        let out = if matches!(next, AbcState::Reject) {
            AbcOutput::Reject
        } else {
            AbcOutput::Accept
        };
        (next, out)
    }
}
