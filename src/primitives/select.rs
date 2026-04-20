use crate::core::StateMachine;

/// Projects the `k`th component out of a `Vec`-shaped input. Matches the
/// chapter's `Select(k)` primitive. The input must have at least `k+1`
/// elements; callers are expected to guarantee that.
pub struct Select<T> {
    k: usize,
    _phantom: core::marker::PhantomData<T>,
}

impl<T> Select<T> {
    pub fn new(k: usize) -> Self {
        Self { k, _phantom: core::marker::PhantomData }
    }
}

impl<T: Clone> StateMachine for Select<T> {
    type Input = Vec<T>;
    type Output = T;
    type State = ();

    fn start_state(&self) {}

    fn next_values(&self, _: &(), input: &Vec<T>) -> ((), T) {
        ((), input[self.k].clone())
    }
}
