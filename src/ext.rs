//! Builder-style extension trait for composing machines fluently.
//!
//! ```ignore
//! use state_machines_rs::{SMExt, primitives::{Delay, Increment}};
//!
//! // Python: sm.Feedback(sm.Cascade(Increment(1), sm.Delay(Some(0))))
//! let counter = Increment::new(Some(1))
//!     .cascade(Delay::new(Some(0)))
//!     .feedback();
//! ```

use crate::combinators::{
    Cascade, Feedback, Feedback2, FeedbackAdd, FeedbackSubtract, If, Mux, Parallel, Parallel2,
    ParallelAdd, Switch,
};
use crate::core::StateMachine;
use crate::defined::{Defined, SafeAdd, SafeSub};

/// Fluent composition helpers for [`StateMachine`].
pub trait SMExt: StateMachine + Sized {
    /// Chain another machine after this one — `m1.cascade(m2)` routes this
    /// machine's output into `m2`'s input.
    fn cascade<M2>(self, other: M2) -> Cascade<Self, M2>
    where
        M2: StateMachine<Input = Self::Output>,
    {
        Cascade::new(self, other)
    }

    /// Run both machines on the same input; emit a pair of outputs.
    fn parallel<M2>(self, other: M2) -> Parallel<Self, M2>
    where
        M2: StateMachine<Input = Self::Input>,
    {
        Parallel::new(self, other)
    }

    /// Dual-input parallel: the composite takes `(i1, i2)` and routes
    /// components independently.
    fn parallel2<M2>(self, other: M2) -> Parallel2<Self, M2>
    where
        Self::Input: Defined,
        M2: StateMachine,
        M2::Input: Defined,
    {
        Parallel2::new(self, other)
    }

    /// Run both machines on the same input; sum their outputs.
    fn parallel_add<M2>(self, other: M2) -> ParallelAdd<Self, M2>
    where
        M2: StateMachine<Input = Self::Input, Output = Self::Output>,
        Self::Output: SafeAdd,
    {
        ParallelAdd::new(self, other)
    }

    /// Close the loop: this machine's output feeds back into its input.
    /// Requires input and output types match and support [`Defined`]; also
    /// requires no direct input-to-output dependence (checked at runtime).
    fn feedback<T>(self) -> Feedback<Self>
    where
        Self: StateMachine<Input = T, Output = T>,
        T: Defined,
    {
        Feedback::new(self)
    }

    /// Two-input feedback: external input paired with the feedback value.
    fn feedback2<I, T>(self) -> Feedback2<Self>
    where
        Self: StateMachine<Input = (I, T), Output = T>,
        I: Clone,
        T: Defined,
    {
        Feedback2::new(self)
    }

    /// Feedback-with-sum: `m1_input = external + m2(m1_output)`.
    fn feedback_add<M2, T>(self, m2: M2) -> FeedbackAdd<Self, M2>
    where
        Self: StateMachine<Input = T, Output = T>,
        M2: StateMachine<Input = T, Output = T>,
        T: Defined + SafeAdd,
    {
        FeedbackAdd::new(self, m2)
    }

    /// Feedback-with-difference: `m1_input = external - m2(m1_output)`.
    fn feedback_subtract<M2, T>(self, m2: M2) -> FeedbackSubtract<Self, M2>
    where
        Self: StateMachine<Input = T, Output = T>,
        M2: StateMachine<Input = T, Output = T>,
        T: Defined + SafeSub,
    {
        FeedbackSubtract::new(self, m2)
    }

    /// Dynamic routing: `cond(input) ? self : other` each step, only the
    /// selected branch advances.
    fn switch<M2, F>(self, cond: F, other: M2) -> Switch<Self, M2, F>
    where
        M2: StateMachine<Input = Self::Input, Output = Self::Output>,
        F: Fn(&Self::Input) -> bool,
    {
        Switch::new(cond, self, other)
    }

    /// Always advance both; `cond` selects which output to emit.
    fn mux<M2, F>(self, cond: F, other: M2) -> Mux<Self, M2, F>
    where
        M2: StateMachine<Input = Self::Input, Output = Self::Output>,
        F: Fn(&Self::Input) -> bool,
    {
        Mux::new(cond, self, other)
    }

    /// One-shot decision on the first input: commits to `self` or `other` for
    /// the rest of time.
    fn if_else<M2, F>(self, cond: F, other: M2) -> If<Self, M2, F>
    where
        M2: StateMachine<Input = Self::Input, Output = Self::Output>,
        F: Fn(&Self::Input) -> bool,
    {
        If::new(cond, self, other)
    }
}

impl<M: StateMachine + Sized> SMExt for M {}
