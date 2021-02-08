//! Flat motion profile
//!
//! See [`Flat`].

use core::ops;

use fixed::FixedU32;

use crate::MotionProfile;

/// Flat motion profile
///
/// This is the simplest possible motion profile, as it produces just a constant
/// velocity. Please note that this is of limited use, and should probably be
/// restricted to testing.
///
/// Theoretically, this profile produces infinite acceleration/deceleration at
/// the beginning and end of the movement. In practice, you might get away with
/// this, if the velocity and the load on the motor are low enough. Otherwise,
/// this will definitely produce missed steps.
///
/// Create an instance of this struct using [`Flat::new`], then use the API
/// defined by [`MotionProfile`] (which this struct implements) to generate the
/// acceleration ramp.
///
/// # Unit of Time
///
/// This code is agnostic on which unit of time is used. If you provide the
/// target velocity in steps per second, the unit of the delay returned will be
/// seconds.
///
/// This allows you to pass the target velocity in steps per number of timer
/// counts for the timer you're using, completely eliminating any conversion
/// overhead for the delay.
///
/// # Type Parameter
///
/// The type parameter `Num` defines the type that is used to represent the
/// target velocity and the delay per step. It is set to a 32-bit fixed-point
/// number type by default.
///
/// This default is appropriate for 32-bit microcontrollers, but it might not
/// be ideal for 8- or 16-bit microcontrollers, or target platforms where
/// hardware support for floating point numbers is available. You can override
/// it with other types from the `fixed` crate, or `f32`/`f64`, for example.
pub struct Flat<Num = DefaultNum> {
    delay: Num,
}

impl<Num> Flat<Num>
where
    Num: num_traits::One + ops::Div<Output = Num>,
{
    /// Create a `Flat` instance by passing a target velocity
    ///
    /// The target velocity is specified in steps per unit of time (see
    /// top-level documentation of this struct) and must not be zero.
    ///
    /// # Panics
    ///
    /// Panics, if `target_velocity` is zero.
    pub fn new(target_velocity: Num) -> Self {
        let delay = Num::one() / target_velocity;
        Self { delay }
    }
}

impl<Num> MotionProfile<Num> for Flat<Num>
where
    Num: Copy,
{
    type Iter = Iter<Num>;

    /// Generate the acceleration ramp
    ///
    /// The `num_steps` argument defines the number of steps to take. Returns an
    /// iterator that yields one delay value per step, and `None` after that.
    ///
    /// Since this is the flat motion profile, all delay values yielded will be
    /// the same (as defined by the target velocity passed to the constructor).
    fn ramp(&self, num_steps: usize) -> Self::Iter {
        Iter {
            delay: self.delay,

            step: 0,
            num_steps,
        }
    }
}

/// The iterator returned by [`Flat`]
///
/// See [`Flat`]'s [`MotionProfile::ramp`] implementation
pub struct Iter<Num> {
    delay: Num,

    step: usize,
    num_steps: usize,
}

impl<Num> Iterator for Iter<Num>
where
    Num: Copy,
{
    type Item = Num;

    fn next(&mut self) -> Option<Self::Item> {
        if self.step >= self.num_steps {
            return None;
        }

        self.step += 1;

        Some(self.delay)
    }
}

/// The default numeric type used by [`Flat`]
pub type DefaultNum = FixedU32<typenum::U16>;

#[cfg(test)]
mod tests {
    use crate::{Flat, MotionProfile as _};

    #[test]
    fn flat_should_produce_correct_number_of_steps() {
        let flat = Flat::new(2.0); // steps per second

        let num_steps = 200;
        assert_eq!(flat.ramp(num_steps).count(), num_steps);
    }

    #[test]
    fn flat_should_produce_constant_speed() {
        let flat = Flat::new(2.0); // steps per second

        for delay in flat.ramp(200) {
            assert_eq!(delay, 0.5);
        }
    }
}
