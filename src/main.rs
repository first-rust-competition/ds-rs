#![allow(dead_code)]

#[macro_use]
extern crate bitflags;

mod outbound;
mod inbound;
mod ds;
pub mod util;

use std::thread;
use std::time::Duration;

use self::outbound::udp::types::Alliance;
use self::ds::DriverStation;

pub type Result<T> = std::result::Result<T, failure::Error>;

fn main() {
    let mut ds = DriverStation::new(Alliance::new_red(1), 4069);

    loop {
        thread::sleep(Duration::from_secs(10))
    }
}
