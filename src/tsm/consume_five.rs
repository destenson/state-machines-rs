use crate::core::StateMachine;
use crate::defined::SafeAdd;

/// Reads five inputs, emits `None` for the first four and the running total on
/// the fifth, then terminates. §4.3.
pub struct ConsumeFiveValues<T> {
    _phantom: core::marker::PhantomData<T>,
}

impl<T> ConsumeFiveValues<T> {
    pub fn new() -> Self {
        Self { _phantom: core::marker::PhantomData }
    }
}

impl<T> Default for ConsumeFiveValues<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> StateMachine for ConsumeFiveValues<T>
where
    T: SafeAdd + Clone + Default,
{
    type Input = T;
    type Output = Option<T>;
    /// `(count, running_total)`.
    type State = (usize, T);

    fn start_state(&self) -> (usize, T) {
        (0, T::default())
    }

    fn next_values(&self, state: &(usize, T), input: &T) -> ((usize, T), Option<T>) {
        let (count, total) = state;
        let new_total = total.safe_add(input);
        let new_state = (count + 1, new_total.clone());
        let output = if *count == 4 { Some(new_total) } else { None };
        (new_state, output)
    }

    fn done(&self, state: &(usize, T)) -> bool {
        state.0 == 5
    }
}
