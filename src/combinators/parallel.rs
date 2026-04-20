use crate::core::StateMachine;
use crate::defined::{Defined, SafeAdd};

/// Runs `m1` and `m2` on the same input; output is the pair `(o1, o2)`.
/// §4.2.2.
pub struct Parallel<M1, M2> {
    m1: M1,
    m2: M2,
}

impl<M1, M2> Parallel<M1, M2> {
    pub fn new(m1: M1, m2: M2) -> Self {
        Self { m1, m2 }
    }
}

impl<M1, M2, I> StateMachine for Parallel<M1, M2>
where
    M1: StateMachine<Input = I>,
    M2: StateMachine<Input = I>,
{
    type Input = I;
    type Output = (M1::Output, M2::Output);
    type State = (M1::State, M2::State);

    fn start_state(&self) -> Self::State {
        (self.m1.start_state(), self.m2.start_state())
    }

    fn next_values(&self, state: &Self::State, input: &I) -> (Self::State, Self::Output) {
        let (s1, s2) = state;
        let (ns1, o1) = self.m1.next_values(s1, input);
        let (ns2, o2) = self.m2.next_values(s2, input);
        ((ns1, ns2), (o1, o2))
    }
}

/// Like [`Parallel`], but takes a pair `(i1, i2)` and routes each component to
/// a different machine. Accepts an undefined input (splits it into a pair of
/// undefined) so it composes with [`Feedback`](super::Feedback). §4.2.2.
pub struct Parallel2<M1, M2> {
    m1: M1,
    m2: M2,
}

impl<M1, M2> Parallel2<M1, M2> {
    pub fn new(m1: M1, m2: M2) -> Self {
        Self { m1, m2 }
    }
}

impl<M1, M2, I1, I2> StateMachine for Parallel2<M1, M2>
where
    M1: StateMachine<Input = I1>,
    M2: StateMachine<Input = I2>,
    I1: Defined,
    I2: Defined,
{
    type Input = (I1, I2);
    type Output = (M1::Output, M2::Output);
    type State = (M1::State, M2::State);

    fn start_state(&self) -> Self::State {
        (self.m1.start_state(), self.m2.start_state())
    }

    fn next_values(
        &self,
        state: &Self::State,
        input: &(I1, I2),
    ) -> (Self::State, Self::Output) {
        let (s1, s2) = state;
        let (i1, i2) = input;
        let (ns1, o1) = self.m1.next_values(s1, i1);
        let (ns2, o2) = self.m2.next_values(s2, i2);
        ((ns1, ns2), (o1, o2))
    }
}

/// Runs `m1` and `m2` on the same input; output is `o1 + o2` (via
/// [`SafeAdd`]). §4.2.2.
pub struct ParallelAdd<M1, M2> {
    m1: M1,
    m2: M2,
}

impl<M1, M2> ParallelAdd<M1, M2> {
    pub fn new(m1: M1, m2: M2) -> Self {
        Self { m1, m2 }
    }
}

impl<M1, M2, I, O> StateMachine for ParallelAdd<M1, M2>
where
    M1: StateMachine<Input = I, Output = O>,
    M2: StateMachine<Input = I, Output = O>,
    O: SafeAdd,
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
        ((ns1, ns2), o1.safe_add(&o2))
    }
}
