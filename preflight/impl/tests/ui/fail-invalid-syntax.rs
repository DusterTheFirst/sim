#![no_std]

use preflight_impl::{avionics_harness, Avionics, Control, Sensors};

#[derive(Debug)]
struct Controller;

#[avionics_harness(ooooo , eee / 0099 | gee)]
impl Avionics for Controller {
    fn guide(&mut self, _: &Sensors) -> Option<Control> {
        todo!()
    }
}

fn main() {}
