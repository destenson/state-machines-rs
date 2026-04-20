//! Terminating state machines (TSMs) and their combinators.
//!
//! A TSM is any [`StateMachine`] whose `done` method can return `true`.
//! Sequentially composing TSMs is trickier than composing ordinary SMs because
//! each element in the sequence has its own state type; a `Vec` of
//! heterogeneous machines can't be held by a single concrete `State` type.
//!
//! The resolution here: TSM combinators (`Repeat`, `Sequence`, `Until`,
//! `RepeatUntil`) are themselves stateful — they own their inner machines and
//! those machines' states, and expose the [`DynTSM`] interface rather than
//! implementing `StateMachine` directly. Primitive TSMs still implement the
//! pure `StateMachine` trait; wrap them with [`Stateful::new`] to feed them
//! into a TSM combinator.
//!
//! The chapter's pedagogical TSMs (`CharTSM`, `ConsumeFiveValues`) are
//! gated behind the `toy` feature. Writing a small `impl StateMachine`
//! with a custom `done` predicate is typically as short as instantiating
//! one of those toys, so the default surface stays free of chapter-only
//! primitives.

#[cfg(feature = "toy")]
mod char_tsm;
#[cfg(feature = "toy")]
mod consume_five;
mod repeat;
mod sequence;
mod until;

#[cfg(feature = "toy")]
pub use char_tsm::CharTSM;
#[cfg(feature = "toy")]
pub use consume_five::ConsumeFiveValues;
pub use repeat::Repeat;
pub use sequence::Sequence;
pub use until::{RepeatUntil, Until};

use crate::core::StateMachine;

/// Stateful, object-safe view of a terminating state machine. TSM combinators
/// store `Box<dyn DynTSM<I, O>>` so they can hold heterogeneous inner
/// machines.
pub trait DynTSM<I, O> {
    /// Reset to the starting state. Called by [`Repeat`] between iterations.
    fn reset(&mut self);

    /// Advance one step on the given input and return the output.
    fn step(&mut self, input: &I) -> O;

    /// Has the machine terminated?
    fn is_done(&self) -> bool;
}

/// Adapter: wrap any [`StateMachine`] with its current state so it can be used
/// where a [`DynTSM`] is expected.
pub struct Stateful<M: StateMachine> {
    machine: M,
    state: M::State,
}

impl<M: StateMachine> Stateful<M> {
    pub fn new(machine: M) -> Self {
        let state = machine.start_state();
        Self { machine, state }
    }

    pub fn state(&self) -> &M::State {
        &self.state
    }
}

impl<M> DynTSM<M::Input, M::Output> for Stateful<M>
where
    M: StateMachine,
{
    fn reset(&mut self) {
        self.state = self.machine.start_state();
    }

    fn step(&mut self, input: &M::Input) -> M::Output {
        let (new_state, output) = self.machine.next_values(&self.state, input);
        self.state = new_state;
        output
    }

    fn is_done(&self) -> bool {
        self.machine.done(&self.state)
    }
}

/// Convenience: box a [`StateMachine`] as a `Box<dyn DynTSM>` for feeding into
/// TSM combinators.
pub fn into_dyn<M>(machine: M) -> Box<dyn DynTSM<M::Input, M::Output>>
where
    M: StateMachine + 'static,
{
    Box::new(Stateful::new(machine))
}
