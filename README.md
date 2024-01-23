# RampMaker - Stepper Acceleration Ramp Generator

[![crates.io](https://img.shields.io/crates/v/ramp-maker.svg)](https://crates.io/crates/ramp-maker) [![Documentation](https://docs.rs/ramp-maker/badge.svg)](https://docs.rs/ramp-maker) ![CI Build](https://github.com/flott-motion/ramp-maker/workflows/CI%20Build/badge.svg)

## About

RampMaker provides implementations of stepper motor acceleration profiles, as well as a trait to abstract over them. Right now only a trapezoidal profile (plus a flat profile for testing) is supported.

Also check out [Stepper], the universal stepper motor interface. If you're looking for an alternative to this library, you might like [stepgen].


## Status

Active development on Stepper has ceased, but the project is still passively maintained.

The library is usable, but far from mature.


## Usage

RampMaker is a library written in Rust and designed for use in Rust projects. It will run on any platform supported by Rust, including microcontrollers.

Add RampMaker to your `Cargo.toml` like this:

``` toml
[dependencies.ramp-maker]
version = "0.2" # always use the latest version here
```


## License

This project is open source software, licensed under the terms of the [Zero Clause BSD License] (0BSD, for short). This basically means you can do anything with the software, without any restrictions, but you can't hold the authors liable for problems.

See [LICENSE.md] for full details.


[Stepper]: https://crates.io/crates/stepper
[stepgen]: https://crates.io/crates/stepgen
[Zero Clause BSD License]: https://opensource.org/licenses/0BSD
[LICENSE.md]: https://github.com/flott-motion/ramp-maker/blob/main/LICENSE.md
