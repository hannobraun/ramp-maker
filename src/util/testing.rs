//! Reusable testing code for motion profiles
//!
//! This module contains tests that need to hold for all [`MotionProfile`]
//! implementations. It is called from the test suites of each implementation in
//! this crate, and may be reused by other implementations from outside this
//! crate.
//!
//! Each test is added as a function here, but all of them are called from the
//! [`test`] function.

#![cfg(test)]

use crate::MotionProfile;

/// Run full test suite on the provided [`MotionProfile`] implementation
pub fn test<Profile>()
where
    Profile: MotionProfile + Default,
{
    position_mode_must_produce_correct_number_of_steps(Profile::default());
}

/// A motion in position mode must produce the correct number of steps
pub fn position_mode_must_produce_correct_number_of_steps(
    profile: impl MotionProfile,
) {
    let num_steps = 200;
    assert_eq!(profile.ramp(num_steps).count() as u32, num_steps);
}
