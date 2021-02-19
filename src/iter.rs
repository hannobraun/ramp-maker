//! Iterators used in conjunction with [`MotionProfile`]

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
