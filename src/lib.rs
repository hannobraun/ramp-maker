//! RampMaker - Stepper Acceleration Ramp Generator
//!
//! RampMaker is a library that generates motion profiles for stepper motors. It
//! can be used independently, or together with [Step/Dir].
//!
//! Trinamic have [an overview over motion profiles][overview] on their website.
//!
//! # Cargo Features
//!
//! This library works without the standard library (`no_std`) by default. This
//! limits support for `f32`/`f64` for motion profiles that need to compute a
//! square root, as this operation is not available in the core library (if
//! you're using the default fixed-point types, you're not affected by this).
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
pub mod util;

pub use self::{flat::Flat, trapezoidal::Trapezoidal};

/// Abstract interface for motion profiles
///
/// Implemented by all motion profiles in this library. Can be used to
/// write abstract code that doesn't care about the specific motion profile
/// used.
pub trait MotionProfile {
    /// The type used for representing velocities
    type Velocity;

    /// The type used for representing delay values
    type Delay;

    /// The iterator returned by [`MotionProfile::ramp`]
    type Iter: Iterator<Item = Self::Delay>;

    /// Enter position mode
    ///
    /// In position mode, the motion profile will attempt to move for a specific
    /// number of steps and come to a stand-still at the target step.
    ///
    /// The number of steps given here is always relative to the current
    /// position, as implementations of this trait are not expected to keep
    /// track of an absolute position.
    fn enter_position_mode(
        &mut self,
        max_velocity: Self::Velocity,
        num_steps: u32,
    );

    /// Generate the acceleration ramp
    ///
    /// The returned iterator yields one value per step, as defined by the call
    /// to [`enter_position_mode`], each value defining a delay between two
    /// steps.
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
    ///
    /// [`enter_position_mode`]: MotionProfile::enter_position_mode
    fn ramp(&self) -> Self::Iter;
}
