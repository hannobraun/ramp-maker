//! Flat motion profile
//!
//! See [`Flat`].

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
    delay: Option<Num>,
    num_steps: u32,
}

impl<Num> Flat<Num> {
    /// Create a new instance of `Flat`
    pub fn new() -> Self {
        Self {
            delay: None,
            num_steps: 0,
        }
    }
}

impl Default for Flat<f32> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Num> MotionProfile for Flat<Num>
where
    Num: Copy + num_traits::Inv<Output = Num>,
{
    type Velocity = Num;
    type Delay = Num;
    type Iter = Iter<Num>;

    fn enter_position_mode(
        &mut self,
        max_velocity: Self::Velocity,
        num_steps: u32,
    ) {
        self.delay = Some(max_velocity.inv());
        self.num_steps = num_steps;
    }

    /// Generate the acceleration ramp
    ///
    /// The `num_steps` argument defines the number of steps to take. Returns an
    /// iterator that yields one delay value per step, and `None` after that.
    ///
    /// Since this is the flat motion profile, all delay values yielded will be
    /// the same (as defined by the target velocity passed to the constructor).
    fn ramp(&self) -> Self::Iter {
        Iter {
            // This will panic, if `enter_position_mode` hasn't been called
            // first. Typically I'd at least mention this in the method
            // documentation, but this is only temporary, while I work on
            // transitioning to a more flexible API.
            delay: self.delay.unwrap(),
            num_steps: self.num_steps,
        }
    }
}

/// The iterator returned by [`Flat`]
///
/// See [`Flat`]'s [`MotionProfile::ramp`] implementation
pub struct Iter<Num> {
    delay: Num,
    num_steps: u32,
}

impl<Num> Iterator for Iter<Num>
where
    Num: Copy,
{
    type Item = Num;

    fn next(&mut self) -> Option<Self::Item> {
        if self.num_steps == 0 {
            return None;
        }

        self.num_steps -= 1;

        Some(self.delay)
    }
}

/// The default numeric type used by [`Flat`]
pub type DefaultNum = FixedU32<typenum::U16>;

#[cfg(test)]
mod tests {
    use crate::{Flat, MotionProfile as _};

    #[test]
    fn flat_should_pass_motion_profile_tests() {
        crate::util::testing::test::<Flat<f32>>();
    }

    #[test]
    fn flat_should_produce_constant_velocity() {
        let mut flat = Flat::new();

        flat.enter_position_mode(2.0, 200);
        for delay in flat.ramp() {
            assert_eq!(delay, 0.5);
        }
    }
}
