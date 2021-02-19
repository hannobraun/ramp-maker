//! Trapezoidal motion profile
//!
//! See [`Trapezoidal`].

use core::ops;

use az::Az as _;
use num_traits::{clamp_max, clamp_min};

use crate::{
    util::traits::{Ceil, Sqrt},
    MotionProfile,
};

/// Trapezoidal motion profile
///
/// Generates an approximation of a trapezoidal ramp, following the algorithm
/// laid out here:
/// [http://hwml.com/LeibRamp.htm](http://hwml.com/LeibRamp.htm)
///
/// A PDF version of that page is available:
/// [http://hwml.com/LeibRamp.pdf](http://hwml.com/LeibRamp.pdf)
///
/// This implementation makes the following simplifications:
/// - The unit of time used is left to the user (see "Unit of Time" below), so
///   the frequency variable `F` is ignored.
/// - The initial velocity `v0` is assumed to be zero, making this
///   implementation suitable only for starting and stopping at a stand-still.
///
/// Create an instance of this struct using [`Trapezoidal::new`], then use the
/// API defined by [`MotionProfile`] (which this struct implements) to generate
/// the acceleration ramp.
///
/// # Acceleration Ramp
///
/// This struct will generate a trapezoidal acceleration ramp with the following
/// attributes:
/// - The velocity will always be equal to or less than the maximum velocity
///   passed to the constructor.
/// - While ramping up or down, the acceleration will be an approximation
///   of the target acceleration passed to the constructor.
///
/// # Unit of Time
///
/// This code is agnostic on which unit of time is used. If you provide the
/// target acceleration and maximum velocity in steps per second, the unit of
/// the delay returned will be seconds.
///
/// This allows you to pass the parameters in steps per number of timer counts
/// for the timer you're using, completely eliminating any conversion overhead
/// for the delay.
///
/// # Type Parameter
///
/// The type parameter `Num` defines the type that is used to represent the
/// target acceleration, maximum velocity, and delays per step. It is set to a
/// 64-bit fixed-point number type by default.
///
/// You can override the default with `f32`, `f64`, or any other type from the
/// `fixed` crate. Please note that this code uses a very naive approach
/// regarding its use of numeric types, which does not play well lower-accuracy
/// fixed-point types. Please be very careful when using any other other type
/// than the default. The code might not even generate a proper trapezoidal
/// ramp, if accuracy is too low!
///
/// Please note that by default, support for `f32`/`f64` is not available. Check
/// out the section on Cargo features from the documentation in the root module
/// to learn how to enable it.
pub struct Trapezoidal<Num = DefaultNum> {
    delay_min: Num,
    delay_initial: Num,
    target_accel: Num,
}

impl<Num> Trapezoidal<Num>
where
    Num: Copy
        + num_traits::One
        + num_traits::Inv<Output = Num>
        + ops::Add<Output = Num>
        + ops::Div<Output = Num>
        + Sqrt,
{
    /// Create a new instance of `Trapezoidal`
    ///
    /// Accepts two arguments:
    /// - `target_accel`, the target acceleration in steps per (unit of time)^2.
    /// - `max_velocity`, the maximum velocity in steps per unit of time.
    ///
    /// Both parameters must not be zero. See the struct documentation for
    /// information about units of time.
    ///
    /// # Panics
    ///
    /// Panics, if `target_accel` or `max_velocity` are zero.
    pub fn new(target_accel: Num, max_velocity: Num) -> Self {
        // Based on equation [7] in the reference paper.
        let min_delay = max_velocity.inv();

        // Based on equation [17] in the referenced paper.
        let two = Num::one() + Num::one();
        let initial_delay = Num::one() / (two * target_accel).sqrt();

        Self {
            delay_min: min_delay,
            delay_initial: initial_delay,
            target_accel,
        }
    }
}

// Needed for the `MotionProfile` test suite in `crate::util::testing`.
#[cfg(test)]
impl Default for Trapezoidal<f32> {
    fn default() -> Self {
        Self::new(6000.0, 1000.0)
    }
}

impl<Num> MotionProfile for Trapezoidal<Num>
where
    Num: Copy
        + PartialOrd
        + az::Cast<u32>
        + num_traits::One
        + num_traits::Inv<Output = Num>
        + ops::Add<Output = Num>
        + ops::Sub<Output = Num>
        + ops::Mul<Output = Num>
        + ops::Div<Output = Num>
        + Ceil,
{
    type Delay = Num;
    type Iter = Iter<Num>;

    fn ramp(&self, num_steps: u32) -> Self::Iter {
        Iter {
            delay_min: self.delay_min,
            delay_initial: self.delay_initial,
            target_accel: self.target_accel,

            steps_left: num_steps,

            delay_prev: self.delay_initial,
        }
    }
}

/// The iterator returned by [`Trapezoidal`]
///
/// See [`Trapezoidal`]'s [`MotionProfile::ramp`] implementation
pub struct Iter<Num> {
    delay_min: Num,
    delay_initial: Num,
    target_accel: Num,

    steps_left: u32,

    delay_prev: Num,
}

impl<Num> Iterator for Iter<Num>
where
    Num: Copy
        + PartialOrd
        + az::Cast<u32>
        + num_traits::One
        + num_traits::Inv<Output = Num>
        + ops::Add<Output = Num>
        + ops::Sub<Output = Num>
        + ops::Mul<Output = Num>
        + ops::Div<Output = Num>
        + Ceil,
{
    type Item = Num;

    fn next(&mut self) -> Option<Self::Item> {
        if self.steps_left == 0 {
            return None;
        }

        // Compute some basic numbers we're going to need for the following
        // calculations. All of this is statically known, so let's hope it
        // optimizes out.
        let two = Num::one() + Num::one();
        let three = two + Num::one();
        let one_five = three / two;

        let velocity = self.delay_prev.inv();
        let steps_to_stop = (velocity * velocity) / (two * self.target_accel);
        let steps_to_stop = steps_to_stop.ceil().az::<u32>();

        // Compute the delay for the next step. See [22] in the referenced
        // paper.
        //
        // We don't differentiate between acceleration and plateau here, as we
        // clamp the delay value further down anyway, which creates the plateau.
        let q = self.target_accel * self.delay_prev * self.delay_prev;
        let addend = one_five * q * q;
        let delay_next = if self.steps_left > steps_to_stop {
            // Ramping up
            self.delay_prev * (Num::one() - q + addend)
        } else {
            // Ramping down
            self.delay_prev * (Num::one() + q + addend)
        };

        // Ensure that `delay_min <= delay_next <= delay_initial`. See the
        // explanation following [20] in the referenced paper.
        let delay_next = clamp_min(delay_next, self.delay_min);
        let delay_next = clamp_max(delay_next, self.delay_initial);

        self.delay_prev = delay_next;
        self.steps_left -= 1;

        Some(delay_next)
    }
}

/// The default numeric type used by [`Trapezoidal`]
pub type DefaultNum = fixed::FixedU64<typenum::U32>;

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;

    use crate::{MotionProfile as _, Trapezoidal};

    #[test]
    fn trapezoidal_should_pass_motion_profile_tests() {
        crate::util::testing::test::<Trapezoidal<f32>>();
    }

    #[test]
    fn trapezoidal_should_respect_maximum_velocity() {
        let max_velocity = 1000.0; // steps per second
        let trapezoidal = Trapezoidal::new(6000.0, max_velocity);

        let min_delay = 0.001; // seconds
        for delay in trapezoidal.ramp(200) {
            println!("delay: {}, min_delay: {}", delay, min_delay);
            assert!(delay >= min_delay);
        }
    }

    #[test]
    fn trapezoidal_should_come_to_stop_with_last_step() {
        let trapezoidal = Trapezoidal::new(6000.0, 1000.0);

        let mut last_velocity = None;

        for delay in trapezoidal.ramp(200) {
            let velocity = 1.0 / delay;
            println!("Velocity: {}", velocity);
            last_velocity = Some(velocity);
        }

        let last_velocity = last_velocity.unwrap();
        println!("Velocity on last step: {}", last_velocity);

        // No idea if this value is appropriate, but it matches what the
        // algorithm produces. Even if that's not okay, at the very least this
        // test documents the potential shortcoming and protects against further
        // regressions.
        const MIN_VELOCITY: f32 = 200.0;
        assert!(last_velocity <= MIN_VELOCITY);
    }

    #[test]
    fn trapezoidal_should_generate_actual_trapezoidal_ramp() {
        let trapezoidal = Trapezoidal::new(6000.0, 1000.0);

        let mut mode = Mode::RampUp;
        let mut delay_prev = None;

        let mut ramped_up = false;
        let mut plateaued = false;
        let mut ramped_down = false;

        for (i, delay_curr) in trapezoidal.ramp(200).enumerate() {
            if let Some(accel) =
                acceleration_from_delays(&mut delay_prev, delay_curr)
            {
                println!("{}: {}, {:?}", i, accel, mode);

                match mode {
                    Mode::RampUp => {
                        ramped_up = true;

                        if i > 0 && accel == 0.0 {
                            mode = Mode::Plateau;
                        } else {
                            assert!(accel > 0.0);
                        }
                    }
                    Mode::Plateau => {
                        plateaued = true;

                        if accel < 0.0 {
                            mode = Mode::RampDown;
                        } else {
                            assert_eq!(accel, 0.0);
                        }
                    }
                    Mode::RampDown => {
                        ramped_down = true;

                        assert!(accel <= 0.0);
                    }
                }
            }
        }

        assert!(ramped_up);
        assert!(plateaued);
        assert!(ramped_down);
    }

    #[test]
    fn trapezoidal_should_generate_ramp_with_approximate_target_acceleration() {
        let target_accel = 6000.0;
        let trapezoidal = Trapezoidal::new(target_accel, 1000.0);

        // Make the ramp so short that it becomes triangular. This makes testing
        // a bit easier, as we don't have to deal with the plateau.
        let num_steps = 100;

        let mut delay_prev = None;
        for (i, delay_curr) in trapezoidal.ramp(num_steps).enumerate() {
            if let Some(accel) =
                acceleration_from_delays(&mut delay_prev, delay_curr)
            {
                println!("{}: {}, {}", i, target_accel, accel);

                let around_start = i < 5;
                let around_end = i as u32 > num_steps - 5;

                // There are some inaccuracies at various points, which we
                // accept. The rest of the ramp is much more accurate.
                if !around_start && !around_end {
                    assert_abs_diff_eq!(
                        accel.abs(),
                        target_accel,
                        // It's much more accurate for the most part, but can be
                        // quite inaccurate at the beginning and end.
                        epsilon = target_accel * 0.05,
                    );
                }
            }
        }
    }

    #[derive(Debug, Eq, PartialEq)]
    enum Mode {
        RampUp,
        Plateau,
        RampDown,
    }

    /// Computes an acceleration value from two adjacent delays
    fn acceleration_from_delays(
        delay_prev: &mut Option<f32>,
        delay_curr: f32,
    ) -> Option<f32> {
        let mut accel = None;

        if let &mut Some(delay_prev) = delay_prev {
            let velocity_prev = 1.0 / delay_prev;
            let velocity_curr = 1.0 / delay_curr;

            let velocity_diff: f32 = velocity_curr - velocity_prev;

            // - Assumes the velocity defined by a given delay to be reached at
            //   the mid-point between the two steps that the delay separates.
            // - Because of this, the interval between the time of the velocity
            //   being reached and the time of a neighboring step being
            //   initiated, is half the delay (the delay measures the interval
            //   between two steps).
            // - Therefore, the formula below computes the difference between
            //   the points in time at which the two velocities are reached.
            let time_diff = delay_prev / 2.0 + delay_curr / 2.0;

            accel = Some(velocity_diff / time_diff);
        }

        *delay_prev = Some(delay_curr);

        accel
    }
}
