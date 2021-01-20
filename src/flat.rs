use core::{iter, ops};

use fixed::FixedU32;

/// Flat acceleration profile
///
/// This is the simplest possible acceleration profile, as it produces just a
/// constant velocity. Please note that this is of limited use, and should
/// probably be restricted to testing.
///
/// Theoretically, this profile produces infinite acceleration/deceleration at
/// the beginning and end of the movement. In practice, you might get away with
/// this, if the velocity and the load on the motor are low enough. Otherwise,
/// this will definitely produce missed steps.
///
/// # Type Parameter
///
/// The type parameter `Num` defines the type that is used to represent the
/// target velocity and the delay per step. By default, this is set to
/// `fixed::FixedU32<typenum::U8>`, that is a fixed-point number, backed by a
/// `u32`, and 8 fractional bits.
///
/// This default is appropriate for 32-bit microcontrollers, but it might not
/// be ideal for 8- or 16-bit microcontrollers, or target platforms where
/// hardware support for floating point numbers is available. You can override
/// it with other types from the `fixed` crate, or `f32`/`f64`, for example.
pub struct Flat<Num = FixedU32<typenum::U8>> {
    delay: Num,
}

impl<Num> Flat<Num>
where
    Num: Clone + num_traits::One + ops::Div<Output = Num>,
{
    /// Create a `Flat` instance by passing a target velocity
    ///
    /// The target velocity is specified in steps per second and must not be
    /// zero.
    ///
    /// # Panics
    ///
    /// Panics, if `target_velocity` is zero.
    pub fn new(target_speed: Num) -> Self {
        let delay = Num::one() / target_speed;
        Self { delay }
    }

    /// Generate the acceleration ramp
    ///
    /// The `num_steps` argument defines the number of steps to take. Returns an
    /// iterator that yields one delay value per step, and `None` after that.
    ///
    /// Since this is the flat acceleration profile, all delay values yielded
    /// will be the same (as defined by the target speed passed to the
    /// constructor).
    pub fn ramp(&self, num_steps: usize) -> impl Iterator<Item = Num> {
        iter::repeat(self.delay.clone()).take(num_steps)
    }
}

#[cfg(test)]
mod tests {
    use crate::Flat;

    #[test]
    fn flat_should_produce_correct_number_of_steps() {
        let flat = Flat::new(2.0); // steps per second

        let num_steps = 200;
        assert_eq!(flat.ramp(num_steps).count(), num_steps);
    }

    #[test]
    fn flat_should_produce_constant_velocity() {
        let flat = Flat::new(2.0); // steps per second

        for delay in flat.ramp(200) {
            assert_eq!(delay, 0.5);
        }
    }
}
