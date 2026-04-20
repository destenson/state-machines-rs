//! Conditional combinators: [`Switch`], [`Mux`], [`If`]. §4.2.5–4.2.6.

use crate::core::StateMachine;

/// Routes each input to one of two sub-machines based on a predicate. Only the
/// selected machine updates its state on any given step; the other stays put.
pub struct Switch<M1, M2, F> {
    m1: M1,
    m2: M2,
    cond: F,
}

impl<M1, M2, F> Switch<M1, M2, F> {
    pub fn new(cond: F, m1: M1, m2: M2) -> Self {
        Self { m1, m2, cond }
    }
}

impl<M1, M2, F, I, O> StateMachine for Switch<M1, M2, F>
where
    M1: StateMachine<Input = I, Output = O>,
    M2: StateMachine<Input = I, Output = O>,
    F: Fn(&I) -> bool,
{
    type Input = I;
    type Output = O;
    type State = (M1::State, M2::State);

    fn start_state(&self) -> Self::State {
        (self.m1.start_state(), self.m2.start_state())
    }

    fn next_values(&self, state: &Self::State, input: &I) -> (Self::State, O) {
        let (s1, s2) = state;
        if (self.cond)(input) {
            let (ns1, o) = self.m1.next_values(s1, input);
            ((ns1, s2.clone()), o)
        } else {
            let (ns2, o) = self.m2.next_values(s2, input);
            ((s1.clone(), ns2), o)
        }
    }
}

/// Like [`Switch`], but always advances both machines. The predicate only
/// selects which output to emit.
pub struct Mux<M1, M2, F> {
    m1: M1,
    m2: M2,
    cond: F,
}

impl<M1, M2, F> Mux<M1, M2, F> {
    pub fn new(cond: F, m1: M1, m2: M2) -> Self {
        Self { m1, m2, cond }
    }
}

impl<M1, M2, F, I, O> StateMachine for Mux<M1, M2, F>
where
    M1: StateMachine<Input = I, Output = O>,
    M2: StateMachine<Input = I, Output = O>,
    F: Fn(&I) -> bool,
{
    type Input = I;
    type Output = O;
    type State = (M1::State, M2::State);

    fn start_state(&self) -> Self::State {
        (self.m1.start_state(), self.m2.start_state())
    }

    fn next_values(&self, state: &Self::State, input: &I) -> (Self::State, O) {
        let (s1, s2) = state;
        let (ns1, o1) = self.m1.next_values(s1, input);
        let (ns2, o2) = self.m2.next_values(s2, input);
        let out = if (self.cond)(input) { o1 } else { o2 };
        ((ns1, ns2), out)
    }
}

/// State for an [`If`] combinator. Until the first input arrives, the machine
/// is in `Start`; once the branch is chosen, it's one of the two `Running`
/// variants and stays there forever.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IfState<S1, S2> {
    Start,
    RunningM1(S1),
    RunningM2(S2),
}

/// Evaluates a condition on the first input and commits to one of two
/// sub-machines for the rest of time. §4.2.6.
pub struct If<M1, M2, F> {
    m1: M1,
    m2: M2,
    cond: F,
}

impl<M1, M2, F> If<M1, M2, F> {
    pub fn new(cond: F, m1: M1, m2: M2) -> Self {
        Self { m1, m2, cond }
    }
}

impl<M1, M2, F, I, O> StateMachine for If<M1, M2, F>
where
    M1: StateMachine<Input = I, Output = O>,
    M2: StateMachine<Input = I, Output = O>,
    F: Fn(&I) -> bool,
{
    type Input = I;
    type Output = O;
    type State = IfState<M1::State, M2::State>;

    fn start_state(&self) -> Self::State {
        IfState::Start
    }

    fn next_values(&self, state: &Self::State, input: &I) -> (Self::State, O) {
        let resolved = match state {
            IfState::Start => {
                if (self.cond)(input) {
                    IfState::RunningM1(self.m1.start_state())
                } else {
                    IfState::RunningM2(self.m2.start_state())
                }
            }
            other => other.clone(),
        };
        match resolved {
            IfState::RunningM1(s) => {
                let (ns, o) = self.m1.next_values(&s, input);
                (IfState::RunningM1(ns), o)
            }
            IfState::RunningM2(s) => {
                let (ns, o) = self.m2.next_values(&s, input);
                (IfState::RunningM2(ns), o)
            }
            IfState::Start => unreachable!("Start branch resolved above"),
        }
    }
}
