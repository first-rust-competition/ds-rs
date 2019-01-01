//! # ds
//!
//! `ds` is a library that allows for control of FIRST Robotics Competition robots.
//! The protocol supported currently is that of the 2018 season, with only the bare minimum
//! required to control the robot currently consumed. Diagnostic and telemetry information is not decoded and is discarded
//!
//! The core trait for use of the crate is the [`DriverStation`](struct.DriverStation.html) crate. This crate
//! provides an API for connecting and controlling to the roboRIO in an FRC robot. It also allows for users to
//! provide joystick input using arbitrary APIs, and to consume any incoming TCP packets.

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate smallvec;

mod outbound;
mod inbound;
mod ds;
pub(crate) mod util;

pub use self::outbound::udp::types::Alliance;
pub use self::outbound::udp::types::tags::*;
pub use self::ds::{DriverStation, JoystickValue};
pub use self::ds::state::Mode;
pub use self::inbound::tcp::*;

pub type Result<T> = std::result::Result<T, failure::Error>;

