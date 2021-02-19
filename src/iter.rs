//! Iterators used in conjunction with [`MotionProfile`]

use core::{marker::PhantomData, ops};

use num_traits::{Inv as _, One as _};

use crate::MotionProfile;

/// An iterator over delay values
///
/// Can be created by calling [`MotionProfile::delays`].
pub struct Delays<'r, Profile>(pub &'r mut Profile);

impl<'r, Profile> Iterator for Delays<'r, Profile>
where
    Profile: MotionProfile,
{
    type Item = Profile::Delay;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next_delay()
    }
}

/// An iterator over velocity values
///
/// Can be created by calling [`MotionProfile::velocities`].
pub struct Velocities<'r, Profile>(pub &'r mut Profile);

impl<'r, Profile> Iterator for Velocities<'r, Profile>
where
    Profile: MotionProfile,
    Profile::Delay: num_traits::Inv<Output = Profile::Velocity>,
{
    type Item = Profile::Velocity;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next_delay().map(|delay| delay.inv())
    }
}

/// An iterator over acceleration values
///
/// Can be created by calling [`MotionProfile::accelerations`].
pub struct Accelerations<'r, Profile: MotionProfile, Accel> {
    /// The motion profile
    pub profile: &'r mut Profile,

    /// The previous delay value
    pub delay_prev: Option<Profile::Delay>,

    _accel: PhantomData<Accel>,
}

impl<'r, Profile, Accel> Accelerations<'r, Profile, Accel>
where
    Profile: MotionProfile,
{
    /// Create a new instance of `Accelerations`
    ///
    /// You can call [`MotionProfile::accelerations`] instead.
    pub fn new(profile: &'r mut Profile) -> Self {
        let delay_prev = profile.next_delay();

        Self {
            profile,
            delay_prev,
            _accel: PhantomData,
        }
    }
}

impl<'r, Profile, Accel> Iterator for Accelerations<'r, Profile, Accel>
where
    Profile: MotionProfile,
    Profile::Delay: Copy
        + num_traits::One
        + num_traits::Inv<Output = Profile::Velocity>
        + ops::Add<Output = Profile::Delay>
        + ops::Div<Output = Profile::Delay>,
    Profile::Velocity: ops::Sub<Output = Profile::Velocity>
        + ops::Div<Profile::Delay, Output = Accel>,
{
    type Item = Accel;

    fn next(&mut self) -> Option<Self::Item> {
        let delay_next = self.profile.next_delay()?;

        let mut accel = None;
        if let Some(delay_prev) = self.delay_prev {
            let velocity_prev = delay_prev.inv();
            let velocity_next = delay_next.inv();

            let velocity_diff = velocity_next - velocity_prev;

            let two = Profile::Delay::one() + Profile::Delay::one();

            // - Assumes the velocity defined by a given delay to be reached at
            //   the mid-point between the two steps that the delay separates.
            // - Because of this, the interval between the time of the velocity
            //   being reached and the time of a neighboring step being
            //   initiated, is half the delay (the delay measures the interval
            //   between two steps).
            // - Therefore, the formula below computes the difference between
            //   the points in time at which the two velocities are reached.
            let time_diff = delay_prev / two + delay_next / two;

            accel = Some(velocity_diff / time_diff);
        }

        self.delay_prev = Some(delay_next);

        accel
    }
}
