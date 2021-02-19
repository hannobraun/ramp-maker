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
pub trait MotionProfile: Sized {
    /// The type used for representing velocities
    type Velocity;

    /// The type used for representing delay values
    type Delay;

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

    /// Return the next step delay
    ///
    /// Produces the delay for the next step. The unit of this delay is
    /// implementation-defined. `None` is returned, if no more steps need to be
    /// taken. This happens when reaching the target step in position mode, or
    /// if velocity is set to zero in either position or velocity mode.
    ///
    /// Please note that motion profiles yield one value per step, even though
    /// only n-1 delay values are needed for n steps. The additional delay value
    /// will lead to an unnecessary delay before the first or after the last
    /// step. This was done to make accidental misuse of this trait less likely,
    /// as the most straight-forward use is to make one step per delay value in
    /// a loop.
    ///
    /// All other details of the motion profile are implementation-defined.
    ///
    /// If you need an iterator that produces the step delays, you can get one
    /// by calling [`MotionProfile::delays`], which internally calls this
    /// method.
    fn next_delay(&mut self) -> Option<Self::Delay>;

    /// Return an iterator over delay values
    ///
    /// This is a convenience method that returns an iterator which internally
    /// just calls [`MotionProfile::next_delay`].
    fn delays(&mut self) -> DelayIter<Self> {
        DelayIter(self)
    }
}

/// An iterator over delay values
///
/// Can be created by calling [`MotionProfile::delays`].
pub struct DelayIter<'r, T>(pub &'r mut T);

impl<'r, T> Iterator for DelayIter<'r, T>
where
    T: MotionProfile,
{
    type Item = T::Delay;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next_delay()
    }
}
