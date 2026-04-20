//! Combinators that wire state machines together.
//!
//! * Dataflow: [`Cascade`], [`Parallel`], [`Parallel2`], [`ParallelAdd`]
//! * Feedback: [`Feedback`], [`Feedback2`], [`FeedbackAdd`], [`FeedbackSubtract`]
//! * Conditional: [`Switch`], [`Mux`], [`If`]

mod cascade;
mod conditional;
mod feedback;
mod parallel;

pub use cascade::Cascade;
pub use conditional::{If, IfState, Mux, Switch};
pub use feedback::{Feedback, Feedback2, FeedbackAdd, FeedbackSubtract};
pub use parallel::{Parallel, Parallel2, ParallelAdd};
