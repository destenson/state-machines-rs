//! Declarative table-driven finite-state machine.
//!
//! Supply an initial state and a transition closure `(state, input) -> (state,
//! output)`; you get back a full [`StateMachine`] without writing an `impl`
//! block. This is the arbitrary-output generalization of
//! [`DfaAcceptor`](super::DfaAcceptor) — the acceptor's output is fixed to
//! `bool` driven by an acceptance predicate, whereas `TableFsm` lets you
//! return anything the transition computes.
//!
//! Use it for vending machines, traffic lights, protocol parsers,
//! handshake flows — anywhere you'd otherwise write a tiny `StateMachine`
//! impl whose entire body is one `match`.

use crate::core::StateMachine;

/// Transition-table FSM.
///
/// `S` = state, `I` = input, `O` = output, `F` = the closure supplied at
/// construction time. The struct takes `F` as a generic parameter so the
/// closure is inlined and there's no virtual dispatch cost.
///
/// # Example
///
/// ```rust
/// use state_machines_rs::{Runner, primitives::TableFsm};
///
/// // Three-color traffic light. Cycles on each tick.
/// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// enum Light { Red, Green, Yellow }
///
/// let light = TableFsm::new(Light::Red, |s: &Light, _: &()| {
///     let next = match s {
///         Light::Red    => Light::Green,
///         Light::Green  => Light::Yellow,
///         Light::Yellow => Light::Red,
///     };
///     (next, *s)
/// });
///
/// let mut r = Runner::new(light);
/// let out: Vec<Light> = (0..6).map(|_| r.step(())).collect();
/// assert_eq!(out, [Light::Red, Light::Green, Light::Yellow,
///                  Light::Red, Light::Green, Light::Yellow]);
/// ```
pub struct TableFsm<S, I, O, F>
where
    F: Fn(&S, &I) -> (S, O),
{
    initial: S,
    transition: F,
    _phantom: core::marker::PhantomData<fn(I) -> O>,
}

impl<S, I, O, F> TableFsm<S, I, O, F>
where
    F: Fn(&S, &I) -> (S, O),
{
    pub fn new(initial: S, transition: F) -> Self {
        Self { initial, transition, _phantom: core::marker::PhantomData }
    }
}

impl<S, I, O, F> StateMachine for TableFsm<S, I, O, F>
where
    S: Clone,
    F: Fn(&S, &I) -> (S, O),
{
    type Input = I;
    type Output = O;
    type State = S;

    fn start_state(&self) -> S {
        self.initial.clone()
    }

    fn next_values(&self, state: &S, input: &I) -> (S, O) {
        (self.transition)(state, input)
    }
}
