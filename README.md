# RampMaker - Stepper Acceleration Ramp Generator

[![crates.io](https://img.shields.io/crates/v/ramp-maker.svg)](https://crates.io/crates/ramp-maker) [![Documentation](https://docs.rs/ramp-maker/badge.svg)](https://docs.rs/ramp-maker) ![CI Build](https://github.com/flott-motion/ramp-maker/workflows/CI%20Build/badge.svg)

**Please consider supporting this project financially. More information below.**

## About

RampMaker provides implementations of stepper motor acceleration profiles, as well as a trait to abstract over them. Right now only a trapezoidal profile (plus a flat profile for testing) is supported.

RampMaker is part of the [Flott] motion control toolkit. Also check out [Stepper], the universal stepper motor interface.

If you're looking for an alternative to this library, you might like [stepgen].


## Status

RampMaker is under active development. Its API is going to change, as more features are added and existing ones are improved.

The library is usable, but far from mature. Please open an issue on the GitHub repository, if you find any limitations.

RampMaker is maintained by [@hannobraun].


## Usage

RampMaker is a library written in Rust and designed for use in Rust projects. It will run on any platform supported by Rust, including microcontrollers.

Add RampMaker to your `Cargo.toml` like this:

``` toml
[dependencies.ramp-maker]
version = "0.2" # always use the latest version here
```


## Funding

If you're getting value out of RampMaker or other libraries from the [Flott] toolkit, please consider supporting us financially. Your sponsorship helps to keep the project healthy and moving forward.

[Hanno Braun][@hannobraun], maintainer and original creator of this library, is [accepting sponsorship](https://github.com/sponsors/hannobraun).


## License

This project is open source software, licensed under the terms of the [Zero Clause BSD License] (0BSD, for short). This basically means you can do anything with the software, without any restrictions, but you can't hold the authors liable for problems.

See [LICENSE.md] for full details.


[Flott]: https://flott-motion.org/
[Stepper]: https://crates.io/crates/stepper
[stepgen]: https://crates.io/crates/stepgen
[@hannobraun]: https://github.com/hannobraun
[Zero Clause BSD License]: https://opensource.org/licenses/0BSD
[LICENSE.md]: https://github.com/flott-motion/ramp-maker/blob/main/LICENSE.md
