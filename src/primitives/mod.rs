//! Primitive state machines.
//!
//! The default set covers the reusable building blocks: pure functions
//! (`Gain`, `Increment`, `Wire`, `Negation`, `Adder`, `Multiplier`,
//! `Select`), the delay line (`Delay`, aliased `R`), `Accumulator`,
//! `UpDown`, and the generic `DfaAcceptor` for recognizing regular
//! languages over arbitrary alphabets.
//!
//! Chapter-specific pedagogical machines (`ABC`, `ParkingGate`,
//! `SumLast3`, `Average2`, `UpDown`) are gated behind the `toy` cargo
//! feature. Each is already expressible via a generic primitive
//! (e.g. `UpDown` is `Accumulator<i64>` with a trivial input mapping),
//! or will be once the TODO items land.
//!
//! Numeric primitives are generic over types implementing [`SafeAdd`] /
//! [`SafeMul`]; instantiate with `Option<f64>` when feeding them into a
//! feedback loop so `None` (undefined) propagates correctly.
//!
//! [`SafeAdd`]: crate::SafeAdd
//! [`SafeMul`]: crate::SafeMul

#[cfg(feature = "toy")]
mod abc;
mod accumulator;
mod adder;
#[cfg(feature = "toy")]
mod average2;
mod delay;
mod dfa;
mod gain;
mod increment;
mod markov_chain;
mod multiplier;
mod negation;
#[cfg(feature = "toy")]
mod parking_gate;
mod select;
#[cfg(feature = "toy")]
mod sum_last3;
mod table_fsm;
#[cfg(feature = "toy")]
mod updown;
mod wire;

#[cfg(feature = "toy")]
pub use abc::{ABC, AbcOutput, AbcState};
pub use accumulator::Accumulator;
pub use adder::Adder;
#[cfg(feature = "toy")]
pub use average2::Average2;
pub use delay::Delay;
pub use dfa::DfaAcceptor;
pub use gain::Gain;
pub use increment::Increment;
pub use markov_chain::{MarkovChain, MarkovChainError};
pub use multiplier::Multiplier;
pub use negation::Negation;
#[cfg(feature = "toy")]
pub use parking_gate::{GateCommand, GatePosition, GateState, ParkingGate, ParkingGateInput};
pub use select::Select;
#[cfg(feature = "toy")]
pub use sum_last3::SumLast3;
pub use table_fsm::TableFsm;
#[cfg(feature = "toy")]
pub use updown::{UpDown, UpDownInput};
pub use wire::Wire;

/// Alias for the delay machine, matching the chapter's shorthand `R` used when
/// composing linear time-invariant systems in chapter 5.
pub type R<T> = Delay<T>;
