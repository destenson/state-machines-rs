use crate::core::StateMachine;

/// Serial composition: output of `m1` feeds into `m2`. The composite has
/// `m1`'s input vocabulary and `m2`'s output vocabulary. §4.2.1.
pub struct Cascade<M1, M2> {
    m1: M1,
    m2: M2,
}

impl<M1, M2> Cascade<M1, M2> {
    pub fn new(m1: M1, m2: M2) -> Self {
        Self { m1, m2 }
    }
}

impl<M1, M2, T> StateMachine for Cascade<M1, M2>
where
    M1: StateMachine<Output = T>,
    M2: StateMachine<Input = T>,
{
    type Input = M1::Input;
    type Output = M2::Output;
    type State = (M1::State, M2::State);

    fn start_state(&self) -> Self::State {
        (self.m1.start_state(), self.m2.start_state())
    }

    fn next_values(
        &self,
        state: &Self::State,
        input: &Self::Input,
    ) -> (Self::State, Self::Output) {
        let (s1, s2) = state;
        let (ns1, o1) = self.m1.next_values(s1, input);
        let (ns2, o2) = self.m2.next_values(s2, &o1);
        ((ns1, ns2), o2)
    }
}
