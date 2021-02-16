//! Numeric traits to fill some gaps in the ecosystem
//!
//! Much code in this library is generic over the numeric type it uses, however,
//! there currently seem to be some holes in the ecosystem. [num-traits]
//! basically covers anything we could need, but some of isn't available in a
//! `no_std` context, or can't be implemented by the types from [fixed] for some
//! reason.
//!
//! This module defines some custom traits and provides implementations for
//! `f32`, `f64`, and all types from [fixed].
//!
//! [num-traits]: https://crates.io/crates/num-traits
//! [fixed]: https://crates.io/crates/fixed

use fixed::types::extra::{LeEqU128, LeEqU16, LeEqU32, LeEqU64, LeEqU8};
use fixed_sqrt::{
    traits::{IsEven, LtU128, LtU16, LtU32, LtU64, LtU8},
    FixedSqrt,
};

/// Defines an interface to the square root operation
pub trait Sqrt {
    /// Return the square root of `self`
    ///
    /// This method can't be called `sqrt`, as that would conflict with the
    /// `sqrt` method of `f32` and `f64`, and fully qualified syntax doesn't
    /// work for those, it seems (at least it didn't work for me, right here).
    fn sqrt2(self) -> Self;
}

#[cfg(any(test, feature = "std"))]
mod impl_using_std {
    impl super::Sqrt for f32 {
        fn sqrt2(self) -> Self {
            self.sqrt()
        }
    }

    impl super::Sqrt for f64 {
        fn sqrt2(self) -> Self {
            self.sqrt()
        }
    }
}

#[cfg(all(not(test), not(feature = "std"), feature = "libm"))]
mod impl_using_libm {
    impl super::Sqrt for f32 {
        fn sqrt2(self) -> Self {
            libm::sqrtf(self)
        }
    }

    impl super::Sqrt for f64 {
        fn sqrt2(self) -> Self {
            libm::sqrt(self)
        }
    }
}

macro_rules! impl_fixed {
    ($($num:ident, ($($bound:ident),*);)*) => {
        $(
            impl<U> Sqrt for fixed::$num<U>
            where
                $(U: $bound,)*
            {
                fn sqrt2(self) -> Self {
                    <Self as FixedSqrt>::sqrt(self)
                }
            }
        )*
    };
}

// Can't use a blanket impl, as that would conflict with any other impl that
// anyone might want to provide.
impl_fixed!(
    FixedU8, (LeEqU8);
    FixedU16, (LeEqU16);
    FixedU32, (LeEqU32);
    FixedU64, (LeEqU64);
    FixedU128, (LeEqU128, IsEven);
    FixedI8, (LtU8);
    FixedI16, (LtU16);
    FixedI32, (LtU32);
    FixedI64, (LtU64);
    FixedI128, (LtU128);
);