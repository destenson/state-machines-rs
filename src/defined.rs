//! The `Defined` / `SafeAdd` / `SafeMul` machinery that models the chapter's
//! `'undefined'` sentinel. Feedback combinators probe a machine with an undefined
//! input first to discover its output; primitives must propagate undefined through
//! arithmetic without crashing.
//!
//! We use `Option<T>` as the canonical carrier (`None` = undefined) rather than
//! inventing a sentinel — it already has the right algebraic structure.

/// A type with a distinguished "undefined" value, used by feedback combinators.
///
/// The feedback combinator feeds `Self::undefined()` to probe the inner machine.
/// If the probe returns a non-undefined value, the machine is free of direct
/// input-to-output dependence and feedback is well-defined.
pub trait Defined: Clone {
    fn undefined() -> Self;
    fn is_undefined(&self) -> bool;
}

impl<T: Clone> Defined for Option<T> {
    fn undefined() -> Self {
        None
    }
    fn is_undefined(&self) -> bool {
        self.is_none()
    }
}

impl<A: Defined, B: Defined> Defined for (A, B) {
    fn undefined() -> Self {
        (A::undefined(), B::undefined())
    }
    fn is_undefined(&self) -> bool {
        self.0.is_undefined() && self.1.is_undefined()
    }
}

/// Addition that propagates undefined (`None + x = None`).
pub trait SafeAdd: Sized {
    fn safe_add(&self, other: &Self) -> Self;
}

/// Multiplication that propagates undefined.
pub trait SafeMul: Sized {
    fn safe_mul(&self, other: &Self) -> Self;
}

/// Subtraction that propagates undefined. Used by [`FeedbackSubtract`].
///
/// [`FeedbackSubtract`]: crate::combinators::FeedbackSubtract
pub trait SafeSub: Sized {
    fn safe_sub(&self, other: &Self) -> Self;
}

macro_rules! impl_safe_ops_for_num {
    ($($t:ty),+) => {
        $(
            impl SafeAdd for $t {
                #[inline]
                fn safe_add(&self, other: &Self) -> Self { *self + *other }
            }
            impl SafeMul for $t {
                #[inline]
                fn safe_mul(&self, other: &Self) -> Self { *self * *other }
            }
            impl SafeSub for $t {
                #[inline]
                fn safe_sub(&self, other: &Self) -> Self { *self - *other }
            }
        )+
    };
}

impl_safe_ops_for_num!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64);

impl<T: SafeAdd + Clone> SafeAdd for Option<T> {
    fn safe_add(&self, other: &Self) -> Self {
        match (self, other) {
            (Some(a), Some(b)) => Some(a.safe_add(b)),
            _ => None,
        }
    }
}

impl<T: SafeMul + Clone> SafeMul for Option<T> {
    fn safe_mul(&self, other: &Self) -> Self {
        match (self, other) {
            (Some(a), Some(b)) => Some(a.safe_mul(b)),
            _ => None,
        }
    }
}

impl<T: SafeSub + Clone> SafeSub for Option<T> {
    fn safe_sub(&self, other: &Self) -> Self {
        match (self, other) {
            (Some(a), Some(b)) => Some(a.safe_sub(b)),
            _ => None,
        }
    }
}
