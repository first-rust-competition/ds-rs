use failure::bail;

use std::thread;

mod conn;
pub(crate) mod state;

use self::conn::*;
use self::state::*;

use std::sync::Arc;
use futures::executor::block_on;

use futures_channel::mpsc::{UnboundedSender, unbounded};

use crate::proto::tcp::outbound::{GameData, TcpTag};
use crate::proto::udp::inbound::types::Trace;
use crate::proto::udp::outbound::types::tags::UdpTag;
use crate::proto::udp::outbound::types::*;
use crate::util::ip_from_team_number;
use crate::{Result, TcpPacket};

/// Represents a connection to the roboRIO acting as a driver station
///
/// This struct will contain relevant functions to update the state of the robot,
/// and also manages the threads that manage network connections and joysticks
pub struct DriverStation {
    thread_tx: UnboundedSender<Signal>,
    team_number: u32,
    state: Arc<DsState>,
}

impl DriverStation {
    pub fn new_team(team_number: u32, alliance: Alliance) -> DriverStation {
        Self::new(&ip_from_team_number(team_number), alliance, team_number)
    }

    /// Creates a new driver station for the given alliance station and team number
    /// Connects to the roborio at `ip`. To infer the ip from team_number, use `new_team` instead.
    pub fn new(ip: &str, alliance: Alliance, team_number: u32) -> DriverStation {
        // Channels to communicate to the threads that make up the application, used to break out of infinite loops when the struct is dropped
        let (tx, rx) = unbounded::<Signal>();

        // Global state of the driver station
        let state = Arc::new(DsState::new(alliance));

        // Thread containing UDP sockets communicating with the roboRIO
        let udp_state = state.clone();
        let udp_ip = ip.to_owned();

        thread::spawn(move || {
            use tokio::runtime::Runtime;
            let mut rt = Runtime::new().unwrap();
            rt.block_on(udp_conn(udp_state, udp_ip, rx)).expect("Error with udp connection");
        });

        DriverStation {
            thread_tx: tx,
            state,
            team_number
        }
    }

    /// Provides a closure that will be called when constructing outbound packets to append joystick values
    pub fn set_joystick_supplier(
        &mut self,
        supplier: impl Fn() -> Vec<Vec<JoystickValue>> + Send + Sync + 'static,
    ) {
        block_on(self.state.send().lock()).set_joystick_supplier(supplier);
    }

    pub fn set_tcp_consumer(&mut self, consumer: impl FnMut(TcpPacket) + Send + Sync + 'static) {
        block_on(self.state.tcp().lock()).set_tcp_consumer(consumer);
    }

    /// Changes the alliance for the given `DriverStation`
    pub fn set_alliance(&mut self, alliance: Alliance) {
        block_on(self.state.send().lock()).set_alliance(alliance);
    }

    /// Changes the given `mode` the robot will be in
    pub fn set_mode(&mut self, mode: Mode) {
        block_on(self.state.send().lock()).set_mode(mode);
    }

    pub fn set_team_number(&mut self, team_number: u32) {
        self.team_number = team_number;
        self.thread_tx.unbounded_send(Signal::NewTarget(ip_from_team_number(team_number))).unwrap();
    }

    pub fn team_number(&self) -> u32 {
        self.team_number
    }

    /// Sets the game specific message sent to the robot, and used during the autonomous period
    pub fn set_game_specific_message(&mut self, message: &str) -> Result<()> {
        if message.len() != 3 {
            bail!("Message should be 3 characters long");
        }

        let _ = block_on(self.state
            .tcp()
            .lock())
            .queue_tcp(TcpTag::GameData(GameData {
                gsm: message.to_string(),
            }));
        Ok(())
    }

    /// Returns the current mode of the robot
    pub fn mode(&self) -> Mode {
        *block_on(self.state.send().lock()).mode()
    }

    /// Enables outputs on the robot
    pub fn enable(&mut self) {
        block_on(self.state.send().lock()).enable();
    }

    /// Instructs the roboRIO to restart robot code
    pub fn restart_code(&mut self) {
        block_on(self.state.send().lock()).request(Request::RESTART_CODE);
    }

    /// Instructs the roboRIO to reboot
    pub fn restart_roborio(&mut self) {
        block_on(self.state.send().lock()).request(Request::REBOOT_ROBORIO);
    }

    /// Returns whether the robot is currently enabled
    pub fn enabled(&self) -> bool {
        block_on(self.state.send().lock()).enabled()
    }

    /// Returns the last received Trace from the robot
    pub fn trace(&self) -> Trace {
        block_on(self.state.recv().lock()).trace().clone()
    }

    /// Returns the last received battery voltage from the robot
    pub fn battery_voltage(&self) -> f32 {
        block_on(self.state.recv().lock()).battery_voltage()
    }

    /// Queues a UDP tag to be transmitted with the next outbound packet to the roboRIO
    pub fn queue_udp(&mut self, udp_tag: UdpTag) {
        block_on(self.state.send().lock()).queue_udp(udp_tag);
    }

    /// Returns a Vec of the current contents of the UDP queue
    pub fn udp_queue(&self) -> Vec<UdpTag> {
        block_on(self.state.send().lock()).pending_udp().clone()
    }

    /// Queues a TCP tag to be transmitted to the roboRIO
    pub fn queue_tcp(&mut self, tcp_tag: TcpTag) {
        let _ = block_on(self.state.tcp().lock()).queue_tcp(tcp_tag);
    }

    /// Disables outputs on the robot and disallows enabling it until the code is restarted.
    pub fn estop(&mut self) {
        block_on(self.state.send().lock()).estop();
    }

    /// Returns whether the robot is currently E-stopped
    pub fn estopped(&self) -> bool {
        block_on(self.state.send().lock()).estopped()
    }

    /// Disables outputs on the robot
    pub fn disable(&mut self) {
        block_on(self.state.send().lock()).disable();
    }
}

/// Enum representing a value from a Joystick to be transmitted to the roboRIO
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum JoystickValue {
    /// Represents an axis value to be sent to the roboRIO
    ///
    /// `value` should range from `-1.0..=1.0`, or `0.0..=1.0` if the axis is a trigger
    Axis { id: u8, value: f32 },
    /// Represents a button value to be sent to the roboRIO
    Button { id: u8, pressed: bool },
    /// Represents a POV, or D-pad value to be sent to the roboRIO
    POV { id: u8, angle: i16 },
}

impl Drop for DriverStation {
    fn drop(&mut self) {
        // When this struct is dropped the threads that we spawned should be stopped otherwise we're leaking
        let _ = self.thread_tx.unbounded_send(Signal::Disconnect);
    }
}

#[derive(Debug)]
pub(crate) enum Signal {
    Disconnect,
    NewTarget(String),
}
