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
    delay_min: Option<Num>,
    delay_initial: Num,
    delay_prev: Num,

    target_accel: Num,
    steps_left: u32,
}

impl<Num> Trapezoidal<Num>
where
    Num: Copy
        + num_traits::One
        + ops::Add<Output = Num>
        + ops::Div<Output = Num>
        + Sqrt,
{
    /// Create a new instance of `Trapezoidal`
    ///
    /// Accepts the target acceleration in steps per (unit of time)^2 as an
    /// argument. It must not be zero. See the struct documentation for
    /// information about units of time.
    ///
    /// # Panics
    ///
    /// Panics, if `target_accel` is zero.
    pub fn new(target_accel: Num) -> Self {
        // Based on equation [17] in the referenced paper.
        let two = Num::one() + Num::one();
        let initial_delay = Num::one() / (two * target_accel).sqrt();

        Self {
            delay_min: None,
            delay_initial: initial_delay,
            delay_prev: initial_delay,

            target_accel,
            steps_left: 0,
        }
    }
}

// Needed for the `MotionProfile` test suite in `crate::util::testing`.
#[cfg(test)]
impl Default for Trapezoidal<f32> {
    fn default() -> Self {
        Self::new(6000.0)
    }
}

impl<Num> MotionProfile for Trapezoidal<Num>
where
    Num: Copy
        + PartialOrd
        + az::Cast<u32>
        + num_traits::Zero
        + num_traits::One
        + num_traits::Inv<Output = Num>
        + ops::Add<Output = Num>
        + ops::Sub<Output = Num>
        + ops::Mul<Output = Num>
        + ops::Div<Output = Num>
        + Ceil,
{
    type Velocity = Num;
    type Delay = Num;

    fn enter_position_mode(
        &mut self,
        max_velocity: Self::Velocity,
        num_steps: u32,
    ) {
        // Based on equation [7] in the reference paper.
        self.delay_min = if max_velocity.is_zero() {
            None
        } else {
            Some(max_velocity.inv())
        };

        self.steps_left = num_steps;
    }

    fn next_delay(&mut self) -> Option<Self::Delay> {
        let mode = RampMode::compute(self);

        // Compute some basic numbers we're going to need for the following
        // calculations. All of this is statically known, so let's hope it
        // optimizes out.
        let two = Num::one() + Num::one();
        let three = two + Num::one();
        let one_five = three / two;

        // Compute the delay for the next step. See [22] in the referenced
        // paper.
        let q = self.target_accel * self.delay_prev * self.delay_prev;
        let addend = one_five * q * q;
        let delay_next = match mode {
            RampMode::Idle => {
                return None;
            }
            RampMode::RampUp { delay_min } => {
                let delay_next = self.delay_prev * (Num::one() - q + addend);
                clamp_min(delay_next, delay_min)
            }
            RampMode::Plateau => self.delay_prev,
            RampMode::RampDown => self.delay_prev * (Num::one() + q + addend),
        };

        // See the explanation following [20] in the referenced paper.
        let delay_next = clamp_max(delay_next, self.delay_initial);

        self.delay_prev = delay_next;
        self.steps_left -= 1;

        Some(delay_next)
    }
}

/// The default numeric type used by [`Trapezoidal`]
pub type DefaultNum = fixed::FixedU64<typenum::U32>;

enum RampMode<Num> {
    Idle,
    RampUp { delay_min: Num },
    Plateau,
    RampDown,
}

impl<Num> RampMode<Num>
where
    Num: Copy
        + PartialOrd
        + az::Cast<u32>
        + num_traits::One
        + num_traits::Inv<Output = Num>
        + ops::Add<Output = Num>
        + ops::Div<Output = Num>
        + Ceil,
{
    fn compute(profile: &Trapezoidal<Num>) -> Self {
        // If we don't have a velocity, we can't produce a delay.
        let delay_min = match profile.delay_min {
            Some(delay) => delay,
            None => return Self::Idle,
        };

        if profile.steps_left == 0 {
            return Self::Idle;
        }

        // Compute some basic numbers we're going to need for the following
        // calculations. All of this is statically known, so let's hope it
        // optimizes out.
        let two = Num::one() + Num::one();

        // Compute the number of steps needed to come to a stop. We'll compare
        // that to the number of steps left to the target step below, to
        // determine whether we need to decelerate.
        let velocity = profile.delay_prev.inv();
        let steps_to_stop =
            (velocity * velocity) / (two * profile.target_accel);
        let steps_to_stop = steps_to_stop.ceil().az::<u32>();

        // Determine some key facts about the current situation.
        let target_step_is_close = profile.steps_left <= steps_to_stop;
        let reached_max_velocity = profile.delay_prev <= delay_min;

        if target_step_is_close {
            Self::RampDown
        } else if reached_max_velocity {
            Self::Plateau
        } else {
            Self::RampUp { delay_min }
        }
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;

    use crate::{MotionProfile as _, Trapezoidal};

    #[test]
    fn trapezoidal_should_pass_motion_profile_tests() {
        crate::util::testing::test::<Trapezoidal<f32>>();
    }

    #[test]
    fn trapezoidal_should_come_to_stop_with_last_step() {
        let mut trapezoidal = Trapezoidal::new(6000.0);

        let mut last_velocity = None;

        trapezoidal.enter_position_mode(1000.0, 200);
        for delay in trapezoidal.delays() {
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
        const MIN_VELOCITY: f32 = 110.0;
        assert!(last_velocity <= MIN_VELOCITY);
    }

    #[test]
    fn trapezoidal_should_generate_actual_trapezoidal_ramp() {
        let mut trapezoidal = Trapezoidal::new(6000.0);

        let mut mode = Mode::RampUp;

        let mut ramped_up = false;
        let mut plateaued = false;
        let mut ramped_down = false;

        trapezoidal.enter_position_mode(1000.0, 200);
        for (i, accel) in trapezoidal.accelerations().enumerate() {
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

        assert!(ramped_up);
        assert!(plateaued);
        assert!(ramped_down);
    }

    #[test]
    fn trapezoidal_should_generate_ramp_with_approximate_target_acceleration() {
        let target_accel = 6000.0;
        let mut trapezoidal = Trapezoidal::new(target_accel);

        // Make the ramp so short that it becomes triangular. This makes testing
        // a bit easier, as we don't have to deal with the plateau.
        let num_steps = 100;
        trapezoidal.enter_position_mode(1000.0, num_steps);

        for (i, accel) in trapezoidal.accelerations::<f32>().enumerate() {
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

    #[derive(Debug, Eq, PartialEq)]
    enum Mode {
        RampUp,
        Plateau,
        RampDown,
    }
}
