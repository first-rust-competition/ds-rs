#![allow(dead_code)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate smallvec;

mod outbound;
mod inbound;
mod ds;
pub(crate) mod util;

pub use self::outbound::udp::types::Alliance;
pub use self::ds::DriverStation;
pub use self::ds::state::{Mode, JoystickValue};
pub use self::inbound::tcp::*;

pub type Result<T> = std::result::Result<T, failure::Error>;

