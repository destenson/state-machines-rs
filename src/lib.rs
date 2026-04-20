//! Compositional discrete-time state machines, after MIT 6.01 chapter 4.
//!
//! Every machine implements [`StateMachine`]: a pure `(state, input) -> (state, output)`
//! transition plus a starting state. Mutation lives in [`Runner`]. Machines compose
//! with [`Cascade`], [`Parallel`], [`Feedback`], and friends; terminating machines
//! (TSMs) sequence with [`Repeat`], [`Sequence`], [`Until`], and [`RepeatUntil`].
//!
//! [`Cascade`]: combinators::Cascade
//! [`Parallel`]: combinators::Parallel
//! [`Feedback`]: combinators::Feedback
//! [`Repeat`]: tsm::Repeat
//! [`Sequence`]: tsm::Sequence
//! [`Until`]: tsm::Until
//! [`RepeatUntil`]: tsm::RepeatUntil

pub mod combinators;
pub mod core;
pub mod defined;
pub mod ext;
pub mod primitives;
pub mod tsm;

pub use crate::core::{Runner, StateMachine};
pub use crate::defined::{Defined, SafeAdd, SafeMul, SafeSub};
pub use crate::ext::SMExt;
