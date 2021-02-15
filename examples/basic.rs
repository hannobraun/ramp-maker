// Required to call the `ramp` method.
use ramp_maker::MotionProfile as _;

fn main() {
    // Let's use floating point numbers here to keep the example simple.
    // RampMaker also supports fixed-point numbers though.
    let target_accel = 1000.0; // meters per second^2
    let max_velocity = 1500.0; // meters per second
    let mut profile = ramp_maker::Trapezoidal::new(target_accel, max_velocity);

    let num_steps = 2000;
    profile.enter_position_mode(num_steps);

    for delay in profile.ramp() {
        // How you handle a delay depends on the platform you're running on
        // (RampMaker works pretty much everywhere). Here, we use a fake `Timer`
        // API, to demonstrate how the delays produced by RampMaker must be
        // used.
        let timer = Timer::start(delay);

        // RampMaker doesn't care how you actually interface with the stepper
        // motor, so we use this fake `step` method to demonstrate the
        // principle. If you haven't settled on a solution, why not check out
        // Step/Dir, another library from the Flott toolkit?
        step();

        // Wait until the delay is over before making the next step.
        timer.wait();
    }
}

struct Timer;

impl Timer {
    fn start(_delay_s: f32) -> Self {
        Self
    }

    fn wait(&self) {}
}

fn step() {}
