//! RampMaker - Stepper Acceleration Ramp Generator
//!
//! RampMaker is a library that generates acceleration profiles for stepper
//! motors. It can be used independently, or together with [Step/Dir].
//!
//! # Cargo Features
//!
//! This library works without the standard library (`no_std`) by default. This
//! limits support for `f32`/`f64` for acceleration profiles that need to
//! compute a square root, as this operation is not available in the standard
//! library (if you're using the default fixed-point types, you're not affected
//! by this).
//!
//! If you need full support for `f32`/`f64`, you have the following options:
//! - Enable support for the standard library via the `std` feature. This
//!   obviously only works, if the standard library is available for your
//!   target, and you want to use it.
//! - Enable the `libm` feature. This provides the require square root support
//!   via [libm].
//!
//! [Step/Dir]: https://crates.io/crates/step-dir
//! [libm]: https://crates.io/crates/libm

#![cfg_attr(all(not(test), not(feature = "std")), no_std)]
#![deny(missing_docs, broken_intra_doc_links)]

mod flat;
mod trapezoidal;

pub use self::{flat::Flat, trapezoidal::Trapezoidal};
