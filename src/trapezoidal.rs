//! Trapezoidal motion profile
//!
//! See [`Trapezoidal`].

use core::ops;

use fixed::types::extra::{LeEqU128, LeEqU16, LeEqU32, LeEqU64, LeEqU8};
use fixed_sqrt::{
    traits::{IsEven, LtU128, LtU16, LtU32, LtU64, LtU8},
    FixedSqrt,
};
use num_traits::{clamp_max, clamp_min};

use crate::MotionProfile;

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
/// - None of the optional enhancements are implemented.
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
        let initial_delay = Num::one() / (two * target_accel).sqrt2();

        Self {
            delay_min: min_delay,
            delay_initial: initial_delay,
            target_accel,
        }
    }
}

impl<Num> MotionProfile<Num> for Trapezoidal<Num>
where
    Num: Copy
        + PartialOrd
        + num_traits::One
        + ops::Add<Output = Num>
        + ops::Sub<Output = Num>
        + ops::Mul<Output = Num>,
{
    type Iter = Iter<Num>;

    fn ramp(&self, num_steps: usize) -> Self::Iter {
        Iter {
            delay_min: self.delay_min,
            delay_initial: self.delay_initial,
            target_accel: self.target_accel,

            step: 1,
            num_steps,

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

    step: usize,
    num_steps: usize,

    delay_prev: Num,
}

impl<Num> Iterator for Iter<Num>
where
    Num: Copy
        + PartialOrd
        + num_traits::One
        + ops::Add<Output = Num>
        + ops::Sub<Output = Num>
        + ops::Mul<Output = Num>,
{
    type Item = Num;

    fn next(&mut self) -> Option<Self::Item> {
        if self.step > self.num_steps {
            return None;
        }

        // Compute the delay for the next step. See [20] in the referenced
        // paper.
        //
        // We basically treat our trapezoidal motion profile like a triangular
        // one here. This works because we're actually calculating a triangular
        // profile, as far as this algorithm is concerned. We just turn it into
        // a trapezoidal profile further below, by clamping the delay value
        // before returning it, basically cutting off the top.
        let delay_next = if self.step <= self.num_steps / 2 {
            // Ramping up
            self.delay_prev
                * (Num::one()
                    - self.target_accel * self.delay_prev * self.delay_prev)
        } else {
            // Ramping down
            self.delay_prev
                * (Num::one()
                    + self.target_accel * self.delay_prev * self.delay_prev)
        };

        self.delay_prev = delay_next;
        self.step += 1;

        // Assure that `delay_min <= delay_next <= delay_initial`, for the
        // actually returned delay. See the explanation following [20] in
        // the referenced paper.
        let delay_next = clamp_min(delay_next, self.delay_min);
        let delay_next = clamp_max(delay_next, self.delay_initial);

        Some(delay_next)
    }
}

/// The default numeric type used by [`Trapezoidal`]
pub type DefaultNum = fixed::FixedU64<typenum::U32>;

/// Defines an interface to the square root operation
///
/// The code in this method is generic over the number it uses, however, there
/// currently seems to be a hole in the ecosystem regarding square roots.
/// There's [fixed-sqrt], but it's for numbers from `fixed` only. Then there's
/// `Real` from [num-traits], but this trait is not implemented for the [fixed]
/// types.
///
/// This custom trait fills the gap, by defining a square root method and
/// providing implementations for `f32`, `f64`, and all types from the `fixed`
/// crate.
///
/// [fixed-sqrt]: https://crates.io/crates/fixed-sqrt
/// [num-traits]: https://crates.io/crates/num-traits
/// [fixed]: https://crates.io/crates/fixed
pub trait Sqrt {
    /// Return the square root of `self`
    ///
    /// This method can't be called `sqrt`, as that would conflict with the
    /// `sqrt` method of `f32` and `f64`, and fully qualified syntax doesn't
    /// work for those, it seems (at least it didn't work for me, right here).
    fn sqrt2(self) -> Self;
}

#[cfg(any(test, feature = "std"))]
impl Sqrt for f32 {
    fn sqrt2(self) -> Self {
        self.sqrt()
    }
}

#[cfg(any(test, feature = "std"))]
impl Sqrt for f64 {
    fn sqrt2(self) -> Self {
        self.sqrt()
    }
}

#[cfg(all(not(test), not(feature = "std"), feature = "libm"))]
impl Sqrt for f32 {
    fn sqrt2(self) -> Self {
        libm::sqrtf(self)
    }
}

#[cfg(all(not(test), not(feature = "std"), feature = "libm"))]
impl Sqrt for f64 {
    fn sqrt2(self) -> Self {
        libm::sqrt(self)
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

#[cfg(test)]
mod tests {
    use crate::{MotionProfile as _, Trapezoidal};

    #[test]
    fn trapezoidal_should_produce_correct_number_of_steps() {
        let trapezoidal = Trapezoidal::new(6000.0, 1000.0);

        let num_steps = 200;
        assert_eq!(trapezoidal.ramp(num_steps).count(), num_steps);
    }

    #[test]
    fn trapezoidal_should_respect_maximum_speed() {
        let max_velocity = 1000.0; // steps per second
        let trapezoidal = Trapezoidal::new(6000.0, max_velocity);

        let min_delay = 0.001; // seconds
        for delay in trapezoidal.ramp(200) {
            println!("delay: {}, min_delay: {}", delay, min_delay);
            assert!(delay >= min_delay);
        }
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

                        assert!(accel < 0.0);
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

        let mut previous_mode = None;

        let mut delay_prev = None;
        for (i, delay_curr) in trapezoidal.ramp(200).enumerate() {
            if let Some(accel) =
                acceleration_from_delays(&mut delay_prev, delay_curr)
            {
                println!("{}: {}, {}", i, target_accel, accel);

                let current_mode = Some(Mode::from_accel(accel));

                // Only check acceleration for ramp-up and ramp-down.
                if accel != 0.0 {
                    // It's much more accurate for the most part, but can be
                    // quite inaccurate at the beginning and end.
                    const ALLOWABLE_ERROR: f32 = 0.25;

                    if accel.abs() > target_accel * (1.0 + ALLOWABLE_ERROR) {
                        panic!(
                            "Acceleration too high: {:.0} (target {:.0})",
                            accel, target_accel
                        );
                    }
                    if accel.abs() < target_accel * (1.0 - ALLOWABLE_ERROR) {
                        if previous_mode == Some(Mode::Plateau)
                            && current_mode == Some(Mode::RampDown)
                        {
                            // At the transition from plateau to ramping down,
                            // the acceleration can be much lower than the
                            // target for a single step, due to the way the
                            // algorithm works.
                            //
                            // This is acceptable, so we let it slide here.
                        } else {
                            panic!(
                                "Acceleration too low: {:.0} (target {:.0})",
                                accel, target_accel
                            );
                        }
                    }
                }

                previous_mode = current_mode;
            }
        }
    }

    #[derive(Debug, Eq, PartialEq)]
    enum Mode {
        RampUp,
        Plateau,
        RampDown,
    }

    impl Mode {
        fn from_accel(accel: f32) -> Self {
            match accel {
                accel if accel > 0.0 => Self::RampUp,
                accel if accel == 0.0 => Self::Plateau,
                accel if accel < 0.0 => Self::RampDown,

                accel => {
                    // Must be NaN
                    panic!("Unexpected acceleration: {}", accel);
                }
            }
        }
    }

    /// Computes an acceleration value from two adjacent delays
    fn acceleration_from_delays(
        delay_prev: &mut Option<f32>,
        delay_curr: f32,
    ) -> Option<f32> {
        let mut accel = None;

        if let &mut Some(delay_prev) = delay_prev {
            let speed_prev = 1.0 / delay_prev;
            let speed_curr = 1.0 / delay_curr;

            let speed_diff: f32 = speed_curr - speed_prev;

            // - Assumes the velocity defined by a given delay to be reached at
            //   the mid-point between the two steps that the delay separates.
            // - Because of this, the interval between the time of the velocity
            //   being reached and the time of a neighboring step being
            //   initiated, is half the delay (the delay measures the interval
            //   between two steps).
            // - Therefore, the formula below computes the difference between
            //   the points in time at which the two velocities are reached.
            let time_diff = delay_prev / 2.0 + delay_curr / 2.0;

            accel = Some(speed_diff / time_diff);
        }

        *delay_prev = Some(delay_curr);

        accel
    }
}
