//! RampMaker - Stepper Acceleration Ramp Generator
//!
//! RampMaker is a library that generates acceleration profiles for stepper
//! motors. It can be used independently, or together with [Step/Dir].
//!
//! Trinamic have [an overview over acceleration profiles][overview] on their
//! website.
//!
//! # Cargo Features
//!
//! This library works without the standard library (`no_std`) by default. This
//! limits support for `f32`/`f64` for acceleration profiles that need to
//! compute a square root, as this operation is not available in the core
//! library (if you're using the default fixed-point types, you're not affected
//! by this).
//!
//! If you need full support for `f32`/`f64`, you have the following options:
//! - Enable support for the standard library via the `std` feature. This
//!   obviously only works, if the standard library is available for your
//!   target, and you want to use it.
//! - Enable the `libm` feature. This provides the require square root support
//!   via [libm].
//!
//! [Step/Dir]: https://crates.io/crates/step-dir
//! [overview]: https://www.trinamic.com/technology/motion-control-technology/
//! [libm]: https://crates.io/crates/libm

#![cfg_attr(all(not(test), not(feature = "std")), no_std)]
#![deny(missing_docs, broken_intra_doc_links)]

pub mod flat;
pub mod trapezoidal;

pub use self::{flat::Flat, trapezoidal::Trapezoidal};

/// Abstract interface for acceleration profiles
///
/// Implemented by all acceleration profiles in this library. Can be used to
/// write abstract code that doesn't care about the specific acceleration
/// profile used.
pub trait AccelerationProfile<Num> {
    /// The iterator returned by [`AccelerationProfile::ramp`]
    type Iter: Iterator<Item = Num>;

    /// Generate the acceleration ramp
    ///
    /// `num_steps` defines the number of steps in the acceleration ramp. The
    /// returned iterator yields one value per step, each value defining a delay
    /// between two steps.
    ///
    /// Note that for n steps, only n-1 delay values are actually needed. The
    /// additional delay value will lead to an unnecessary delay before the
    /// first or after the last step. This was done to make accidental misuse of
    /// this method less likely, as the most straight-forward use of this method
    /// is to iterate over all values and make one step per value. If the
    /// additional delay value is relevant for your application, you can just
    /// ignore it.
    ///
    /// All other details of the acceleration ramp, as well as the unit of the
    /// yielded delay values, are implementation-defined.
    fn ramp(&self, num_steps: usize) -> Self::Iter;
}
