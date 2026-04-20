use crate::core::StateMachine;

/// Identity machine: output always equals the current input. §4.2.3.2 exercise 4.7.
pub struct Wire<T> {
    _phantom: core::marker::PhantomData<T>,
}

impl<T> Wire<T> {
    pub fn new() -> Self {
        Self { _phantom: core::marker::PhantomData }
    }
}

impl<T> Default for Wire<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> StateMachine for Wire<T> {
    type Input = T;
    type Output = T;
    type State = ();

    fn start_state(&self) {}

    fn next_values(&self, _: &(), input: &T) -> ((), T) {
        ((), input.clone())
    }
}
