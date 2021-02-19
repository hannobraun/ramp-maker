//! Iterators used in conjunction with [`MotionProfile`]

use crate::MotionProfile;

/// An iterator over delay values
///
/// Can be created by calling [`MotionProfile::delays`].
pub struct Delays<'r, T>(pub &'r mut T);

impl<'r, T> Iterator for Delays<'r, T>
where
    T: MotionProfile,
{
    type Item = T::Delay;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next_delay()
    }
}
