use preflight_impl::{avionics_harness, Avionics, Control, Sensors};

#[derive(Default)]
struct Controller;

#[avionics_harness(arg = "ument")]
impl Avionics for Controller {
    fn guide(&mut self, _: Sensors) -> Control {
        todo!()
    }
}

fn main() {}
