//! Iterators used in conjunction with [`MotionProfile`]

use num_traits::Inv as _;

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
