use crate::detection::*;
use crate::imp::*;
use isvp_sys::IMPIVSInterface;

mod detection;
mod imp;

fn main() {
    env_logger::init();
    unsafe {
        imp_init();
        imp_framesource_init();
        imp_framesource_start();
        detection_init();
        detection_start();
    }
}
