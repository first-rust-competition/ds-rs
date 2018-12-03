#[macro_use]
extern crate bitflags;


mod outbound;
mod inbound;
mod ds;
pub mod util;

use std::thread;

use self::outbound::udp::UdpControlPacket;
use self::outbound::udp::types::*;

use self::inbound::udp::UdpResponsePacket;
use self::ds::DriverStation;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use std::io::{stdin, stdout, Write};

use gilrs::*;


pub type Result<T> = std::result::Result<T, failure::Error>;

fn main() {
    let mut ds = DriverStation::new(Alliance::new_red(1), 4069);
    ds.enable();

    loop {}
}
