//! Primitive state machines from chapter 4.
//!
//! These are the building blocks: pure functions (`Gain`, `Increment`, `Wire`,
//! `Negation`, `Adder`, `Multiplier`, `Select`), simple stateful machines
//! (`Accumulator`, `Delay`, `Average2`, `SumLast3`, `UpDown`), finite-state
//! controllers (`ABC`, `ParkingGate`), and their combinations produce every
//! machine the chapter discusses.
//!
//! Numeric primitives are generic over types implementing [`SafeAdd`] /
//! [`SafeMul`]; instantiate with `Option<f64>` when feeding them into a
//! feedback loop so `None` (undefined) propagates correctly.
//!
//! [`SafeAdd`]: crate::SafeAdd
//! [`SafeMul`]: crate::SafeMul

mod abc;
mod accumulator;
mod adder;
mod average2;
mod delay;
mod gain;
mod increment;
mod multiplier;
mod negation;
mod parking_gate;
mod select;
mod sum_last3;
mod updown;
mod wire;

pub use abc::{ABC, AbcOutput, AbcState};
pub use accumulator::Accumulator;
pub use adder::Adder;
pub use average2::Average2;
pub use delay::Delay;
pub use gain::Gain;
pub use increment::Increment;
pub use multiplier::Multiplier;
pub use negation::Negation;
pub use parking_gate::{GateCommand, GatePosition, GateState, ParkingGate, ParkingGateInput};
pub use select::Select;
pub use sum_last3::SumLast3;
pub use updown::{UpDown, UpDownInput};
pub use wire::Wire;

/// Alias for the delay machine, matching the chapter's shorthand `R` used when
/// composing linear time-invariant systems in chapter 5.
pub type R<T> = Delay<T>;
