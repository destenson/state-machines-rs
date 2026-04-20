//! Deterministic finite-automaton acceptor.
//!
//! Given an initial state, a transition function `(state, input) -> state`,
//! and an acceptance predicate `state -> bool`, this runs as a standard
//! [`StateMachine`] that outputs `true` whenever the current state is
//! accepting.
//!
//! This is the practical generalization of the chapter's
//! [`ABC`](super::ABC) toy — it recognizes any regular language over the
//! input alphabet, given an appropriate transition. Use it as a building
//! block in larger pipelines (e.g. gate a downstream stage with `Switch`
//! on the acceptor's output).

use crate::core::StateMachine;

/// An SM that runs a user-supplied DFA.
///
/// `S` is the state type, `I` is the input alphabet, `F` is the transition,
/// `G` is the acceptance predicate. Once the DFA enters a non-accepting
/// sink state (if the transition routes there), it never escapes — that's
/// how regular-language rejection works.
///
/// # Example
///
/// ```rust
/// use state_machines_rs::{Runner, StateMachine, primitives::DfaAcceptor};
///
/// // Match the language "any string containing the substring ab".
/// // States: 0 = no match yet, 1 = last char was 'a', 2 = matched (sink).
/// let dfa = DfaAcceptor::new(
///     0u8,
///     |s: &u8, c: &char| match (*s, *c) {
///         (2, _) => 2,
///         (_, 'a') => 1,
///         (1, 'b') => 2,
///         _ => 0,
///     },
///     |s: &u8| *s == 2,
/// );
///
/// let out = Runner::new(dfa).transduce("xxabxx".chars());
/// assert_eq!(out, [false, false, false, true, true, true]);
/// ```
pub struct DfaAcceptor<S, I, F, G>
where
    F: Fn(&S, &I) -> S,
    G: Fn(&S) -> bool,
{
    initial: S,
    transition: F,
    accepting: G,
    _input: core::marker::PhantomData<fn(I)>,
}

impl<S, I, F, G> DfaAcceptor<S, I, F, G>
where
    F: Fn(&S, &I) -> S,
    G: Fn(&S) -> bool,
{
    pub fn new(initial: S, transition: F, accepting: G) -> Self {
        Self { initial, transition, accepting, _input: core::marker::PhantomData }
    }
}

impl<S, I, F, G> StateMachine for DfaAcceptor<S, I, F, G>
where
    S: Clone,
    F: Fn(&S, &I) -> S,
    G: Fn(&S) -> bool,
{
    type Input = I;
    type Output = bool;
    type State = S;

    fn start_state(&self) -> S {
        self.initial.clone()
    }

    fn next_values(&self, state: &S, input: &I) -> (S, bool) {
        let next = (self.transition)(state, input);
        let accept = (self.accepting)(&next);
        (next, accept)
    }
}
