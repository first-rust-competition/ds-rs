use crossbeam_channel::{self, Sender, Receiver, unbounded};
use failure::bail;

use std::thread;

pub(crate) mod state;
mod conn;

use self::state::*;
use self::conn::*;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::outbound::udp::types::{Request, Alliance};
use crate::outbound::udp::types::tags::UdpTag;
use crate::outbound::tcp::*;
use crate::inbound::tcp::TcpPacket;
use crate::inbound::udp::types::Trace;
use crate::Result;
use crate::util::ip_from_team_number;

/// Represents a connection to the roboRIO acting as a driver station
///
/// This struct will contain relevant functions to update the state of the robot,
/// and also manages the threads that manage network connections and joysticks
pub struct DriverStation {
    thread_tx: Sender<Signal>,
    thread_rx: Receiver<Signal>,
    state: Arc<Mutex<State>>,
    team_number: u32,
}

impl DriverStation {

    pub fn new_team(team_number: u32, alliance: Alliance) -> DriverStation {
        Self::new(&ip_from_team_number(team_number), team_number, alliance)
    }

    /// Creates a new driver station for the given alliance station and team number
    /// Connects to the roborio at `ip`. To infer the ip from team_number, use `new_team` instead.
    pub fn new(ip: &str, team_number: u32, alliance: Alliance) -> DriverStation {
        // Channels to communicate to the threads that make up the application, used to break out of infinite loops when the struct is dropped
        let (tx, rx) = unbounded::<Signal>();

        // Global state of the driver station
        let state = Arc::new(Mutex::new(State::new(alliance)));

        // Thread containing UDP sockets communicating with the roboRIO
        let udp_state = state.clone();
        let udp_rx = rx.clone();
        let udp_tx = tx.clone();
        let udp_ip = ip.to_owned();
        thread::spawn(move || {
            let monkas_tate = udp_state.clone();
            if udp_thread(udp_state, udp_tx, udp_rx, udp_ip).is_err() {
                let mut state = monkas_tate.lock().unwrap();
                state.set_trace(Trace::empty());
                state.set_battery_voltage(0.0);
            }
        });

        let tcp_state = state.clone();
        let tcp_rx = rx.clone();
        let tcp_ip = ip.to_owned();
        thread::spawn(move || {
            let monkas_tate = tcp_state.clone();
            if tcp_thread(tcp_state, tcp_rx, tcp_ip).is_err() {
                let mut state = monkas_tate.lock().unwrap();
                state.set_trace(Trace::empty());
                state.set_battery_voltage(0.0);
            }
        });

        DriverStation {
            thread_tx: tx,
            thread_rx: rx,
            state,
            team_number,
        }
    }

    /// Queries subthreads to see if this DriverStation is currently connected to a roboRIO
    pub fn connected(&self) -> Result<bool> {
        // Assumption here is that a responding heartbeat implies we're connected.
        self.thread_tx.send(Signal::Heartbeat)?;


        match self.thread_rx.recv_timeout(Duration::from_millis(1000)) {
            Ok(Signal::Heartbeat) => Ok(true),
            Ok(Signal::ConnectTcp) => Ok(true), // Edge case that I'm tried of making this panic
            Err(e) => {
                if e == crossbeam_channel::RecvTimeoutError::Timeout {
                    Ok(false)
                } else {
                    Err(e.into())
                }
            }
            sig => bail!("Unexpected value {:?}", sig)
        }
    }

    /// Provides a closure that will be called when constructing outbound packets to append joystick values
    pub fn set_joystick_supplier(&mut self, supplier: impl Fn() -> Vec<Vec<JoystickValue>> + Send + Sync + 'static) {
        self.state.lock().unwrap().set_joystick_supplier(supplier);
    }

    pub fn set_tcp_consumer(&mut self, consumer: impl FnMut(TcpPacket) + Send + Sync + 'static) {
        self.state.lock().unwrap().set_tcp_consumer(consumer);
    }

    /// Changes the alliance for the given `DriverStation`
    pub fn set_alliance(&mut self, alliance: Alliance) {
        self.state.lock().unwrap().set_alliance(alliance);
    }

    /// Changes the given `mode` the robot will be in
    pub fn set_mode(&mut self, mode: Mode) {
        self.state.lock().unwrap().set_mode(mode);
    }

    /// Sets the game specific message sent to the robot, and used during the autonomous period
    pub fn set_game_specific_message(&mut self, message: &str) -> Result<()> {
        if message.len() != 3 {
            bail!("Message should be 3 characters long");
        }

        self.state.lock().unwrap().queue_tcp(TcpTag::GameData(GameData { gsm: message.to_string() }));
        Ok(())
    }

    /// Returns the current mode of the robot
    pub fn mode(&self) -> Mode {
        *self.state.lock().unwrap().mode()
    }

    /// Enables outputs on the robot
    pub fn enable(&mut self) {
        self.state.lock().unwrap().enable();
    }

    /// Instructs the roboRIO to restart robot code
    pub fn restart_code(&mut self) {
        self.state.lock().unwrap().request(Request::RESTART_CODE);
    }

    /// Instructs the roboRIO to reboot
    pub fn restart_roborio(&mut self) {
        self.state.lock().unwrap().request(Request::REBOOT_ROBORIO);
    }

    /// Returns whether the robot is currently enabled
    pub fn enabled(&self) -> bool {
        *self.state.lock().unwrap().enabled()
    }

    /// Returns the last received Trace from the robot
    pub fn trace(&self) -> Trace {
        self.state.lock().unwrap().trace().clone()
    }

    /// Returns the last received battery voltage from the robot
    pub fn battery_voltage(&self) -> f32 {
        *self.state.lock().unwrap().battery_voltage()
    }

    /// Queues a UDP tag to be transmitted with the next outbound packet to the roboRIO
    pub fn queue_udp(&mut self, udp_tag: UdpTag) {
        self.state.lock().unwrap().queue_udp(udp_tag);
    }

    /// Returns a Vec of the current contents of the UDP queue
    pub fn udp_queue(&self) -> Vec<UdpTag> {
        self.state.lock().unwrap().pending_udp().clone()
    }

    /// Queues a TCP tag to be transmitted to the roboRIO
    pub fn queue_tcp(&mut self, tcp_tag: TcpTag) {
        self.state.lock().unwrap().queue_tcp(tcp_tag);
    }

    /// Returns a Vec of the current contents of the TCP queue
    pub fn tcp_queue(&self) -> Vec<TcpTag> {
        self.state.lock().unwrap().pending_tcp().clone()
    }

    /// Disables outputs on the robot and disallows enabling it until the code is restarted.
    pub fn estop(&mut self) {
        self.state.lock().unwrap().estop();
    }

    /// Returns whether the robot is currently E-stopped
    pub fn estopped(&self) -> bool {
        *self.state.lock().unwrap().estopped()
    }

    /// Disables outputs on the robot
    pub fn disable(&mut self) {
        self.state.lock().unwrap().disable()
    }
}

/// Enum representing a value from a Joystick to be transmitted to the roboRIO
#[derive(Debug)]
pub enum JoystickValue {
    /// Represents an axis value to be sent to the roboRIO
    ///
    /// `value` should range from `-1.0..=1.0`, or `0.0..=1.0` if the axis is a trigger
    Axis {
        id: u8,
        value: f32,
    },
    /// Represents a button value to be sent to the roboRIO
    Button {
        id: u8,
        pressed: bool,
    },
    /// Represents a POV, or D-pad value to be sent to the roboRIO
    POV {
        id: u8,
        angle: i16,
    },
}

impl Drop for DriverStation {
    fn drop(&mut self) {
        // When this struct is dropped the threads that we spawned should be stopped otherwise we're leaking
        self.thread_tx.try_send(Signal::Disconnect).unwrap();
    }
}

#[derive(Debug)]
pub(crate) enum Signal {
    Disconnect,
    ConnectTcp,
    Heartbeat,
}
