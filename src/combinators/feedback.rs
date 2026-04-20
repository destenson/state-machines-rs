//! Feedback combinators.
//!
//! The basic [`Feedback`] wraps a single machine whose output is wired back
//! into its input. The chapter's crucial rule — "the machine must not have a
//! direct dependence of its output on its input" — is what makes this
//! well-defined: feeding an undefined input must still produce a defined
//! output.
//!
//! We express this in Rust by requiring the inner machine's input/output type
//! to implement [`Defined`], then probing with `T::undefined()` to extract the
//! output before closing the loop.

use crate::core::StateMachine;
use crate::defined::{Defined, SafeAdd, SafeSub};

/// Feeds a machine's output back as its input. The composite takes no input
/// (`Input = ()`) and emits the feedback value. §4.2.3.
pub struct Feedback<M> {
    inner: M,
}

impl<M> Feedback<M> {
    pub fn new(inner: M) -> Self {
        Self { inner }
    }
}

impl<M, T> StateMachine for Feedback<M>
where
    M: StateMachine<Input = T, Output = T>,
    T: Defined,
{
    type Input = ();
    type Output = T;
    type State = M::State;

    fn start_state(&self) -> Self::State {
        self.inner.start_state()
    }

    fn next_values(&self, state: &Self::State, _: &()) -> (Self::State, T) {
        let undef = T::undefined();
        let (_, out) = self.inner.next_values(state, &undef);
        debug_assert!(
            !out.is_undefined(),
            "Feedback: inner machine has direct input-to-output dependence; \
             feeding undefined produced undefined"
        );
        let (new_state, _) = self.inner.next_values(state, &out);
        (new_state, out)
    }
}

/// Two-input feedback: the composite has one input; internally the machine
/// sees `(external_input, feedback_value)`. §4.2.3.3.
pub struct Feedback2<M> {
    inner: M,
}

impl<M> Feedback2<M> {
    pub fn new(inner: M) -> Self {
        Self { inner }
    }
}

impl<M, I, T> StateMachine for Feedback2<M>
where
    M: StateMachine<Input = (I, T), Output = T>,
    I: Clone,
    T: Defined,
{
    type Input = I;
    type Output = T;
    type State = M::State;

    fn start_state(&self) -> Self::State {
        self.inner.start_state()
    }

    fn next_values(&self, state: &Self::State, input: &I) -> (Self::State, T) {
        let probe = (input.clone(), T::undefined());
        let (_, out) = self.inner.next_values(state, &probe);
        debug_assert!(
            !out.is_undefined(),
            "Feedback2: inner machine has direct feedback-to-output dependence"
        );
        let closed = (input.clone(), out.clone());
        let (new_state, _) = self.inner.next_values(state, &closed);
        (new_state, out)
    }
}

/// Two machines wired as a feedback loop with a summing junction: input to
/// `m1` is `external + m2(m1_output)`. `m1` must have no direct input-to-output
/// dependence. §4.2.3.4.
pub struct FeedbackAdd<M1, M2> {
    m1: M1,
    m2: M2,
}

impl<M1, M2> FeedbackAdd<M1, M2> {
    pub fn new(m1: M1, m2: M2) -> Self {
        Self { m1, m2 }
    }
}

impl<M1, M2, T> StateMachine for FeedbackAdd<M1, M2>
where
    M1: StateMachine<Input = T, Output = T>,
    M2: StateMachine<Input = T, Output = T>,
    T: Defined + SafeAdd,
{
    type Input = T;
    type Output = T;
    type State = (M1::State, M2::State);

    fn start_state(&self) -> Self::State {
        (self.m1.start_state(), self.m2.start_state())
    }

    fn next_values(&self, state: &Self::State, input: &T) -> (Self::State, T) {
        let (s1, s2) = state;
        let (_, m1_out) = self.m1.next_values(s1, &T::undefined());
        let (ns2, m2_out) = self.m2.next_values(s2, &m1_out);
        let m1_in = input.safe_add(&m2_out);
        let (ns1, _) = self.m1.next_values(s1, &m1_in);
        ((ns1, ns2), m1_out)
    }
}

/// Like [`FeedbackAdd`] but with a subtracting junction: `m1_input = external - m2(m1_output)`.
pub struct FeedbackSubtract<M1, M2> {
    m1: M1,
    m2: M2,
}

impl<M1, M2> FeedbackSubtract<M1, M2> {
    pub fn new(m1: M1, m2: M2) -> Self {
        Self { m1, m2 }
    }
}

impl<M1, M2, T> StateMachine for FeedbackSubtract<M1, M2>
where
    M1: StateMachine<Input = T, Output = T>,
    M2: StateMachine<Input = T, Output = T>,
    T: Defined + SafeSub,
{
    type Input = T;
    type Output = T;
    type State = (M1::State, M2::State);

    fn start_state(&self) -> Self::State {
        (self.m1.start_state(), self.m2.start_state())
    }

    fn next_values(&self, state: &Self::State, input: &T) -> (Self::State, T) {
        let (s1, s2) = state;
        let (_, m1_out) = self.m1.next_values(s1, &T::undefined());
        let (ns2, m2_out) = self.m2.next_values(s2, &m1_out);
        let m1_in = input.safe_sub(&m2_out);
        let (ns1, _) = self.m1.next_values(s1, &m1_in);
        ((ns1, ns2), m1_out)
    }
}
