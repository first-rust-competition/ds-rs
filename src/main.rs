#![feature(nll)]

#[macro_use]
extern crate bitflags;
extern crate byteorder;
extern crate failure;
extern crate chrono;

mod outbound;
mod inbound;
mod ds;
pub mod util;

use std::thread;

use self::outbound::udp::UdpControlPacket;
use self::outbound::udp::types::*;

use self::inbound::udp::UdpResponsePacket;
use self::ds::DriverStation;

use std::time::Duration;

pub type Result<T> = std::result::Result<T, failure::Error>;

fn main() {
    let mut ds = DriverStation::new(Alliance::new_red(1));
//    ds.enable();
//
//    thread::sleep(Duration::from_secs(5));
//    ds.disable();
    loop {
        thread::sleep(Duration::from_secs(5));
    }
}
