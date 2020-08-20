use super::JoystickValue;

use crate::ds::state::recv::{RecvState, TcpState};
use crate::ds::state::send::SendState;
use crate::proto::udp::inbound::types::Status;
use crate::proto::udp::outbound::types::{Alliance, Control};
use crate::TcpPacket;
use std::fmt::Debug;
use tokio::sync::Mutex;

mod recv;
mod send;

type JoystickSupplier = dyn Fn() -> Vec<Vec<JoystickValue>> + Send + Sync + 'static;
type TcpConsumer = dyn FnMut(TcpPacket) + Send + Sync + 'static;

/// The operating mode of the driver station
///
/// Normal operating mode connects to the IP specified by a team number
/// Simulation mode connects to localhost, and is activated by a connection to ::1135
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DsMode {
    Normal,
    Simulation,
}

/// The core state of the driver station, containing locks over all relevant substates
pub struct DsState {
    /// The state associated with the sending UDP socket
    send_state: Mutex<SendState>,
    /// The state associated with the receiving UDP socket
    recv_state: Mutex<RecvState>,
    /// The state associated with the TCP socket
    tcp_state: Mutex<TcpState>,
}

impl DsState {
    pub fn new(alliance: Alliance) -> DsState {
        let send_state = Mutex::new(SendState::new(alliance));
        let recv_state = Mutex::new(RecvState::new());
        let tcp_state = Mutex::new(TcpState::new());

        DsState {
            send_state,
            recv_state,
            tcp_state,
        }
    }

    pub fn send(&self) -> &Mutex<SendState> {
        &self.send_state
    }

    pub fn recv(&self) -> &Mutex<RecvState> {
        &self.recv_state
    }

    pub fn tcp(&self) -> &Mutex<TcpState> {
        &self.tcp_state
    }
}

/// Represents the current Mode that the robot is in. the `Mode` of the robot is considered separately from whether it is enabled or not
#[derive(Copy, Clone, Debug)]
pub enum Mode {
    Autonomous,
    Teleoperated,
    Test,
}

impl Mode {
    /// Decodes the mode of the robot from the given status byte
    pub fn from_status(status: Status) -> Option<Mode> {
        if status & Status::TELEOP == Status::TELEOP {
            Some(Mode::Teleoperated)
        } else if status & Status::AUTO == Status::AUTO {
            Some(Mode::Autonomous)
        } else if status & Status::TEST == Status::TEST {
            Some(Mode::Test)
        } else {
            None
        }
    }

    /// Converts this `Mode` into a `Control` byte that can be modified for encoding the control packet.
    fn to_control(&self) -> Control {
        match *self {
            Mode::Teleoperated => Control::TELEOP,
            Mode::Autonomous => Control::AUTO,
            Mode::Test => Control::TEST,
        }
    }
}
