use crossbeam_channel::{self, Sender, unbounded};

use std::thread;

pub mod state;
mod conn;

use self::state::*;
use self::conn::*;

use std::sync::{Arc, Mutex};

use crate::outbound::udp::types::{Request, Alliance};
use crate::inbound::tcp::TcpPacket;
use crate::inbound::udp::types::Trace;

/// Represents a connection to the roboRIO acting as a driver station
///
/// This struct will contain relevant functions to update the state of the robot,
/// and also manages the threads that manage network connections and joysticks
pub struct DriverStation {
    thread_comm: Sender<Signal>,
    state: Arc<Mutex<State>>,
}

impl DriverStation {
    /// Creates a new driver station for the given alliance station and team number
    pub fn new(alliance: Alliance, team_number: u32) -> DriverStation {
        // Channels to communicate to the threads that make up the application, used to break out of infinite loops when the struct is dropped
        let (tx, rx) = unbounded::<Signal>();

        // Global state of the driver station
        let state = Arc::new(Mutex::new(State::new(alliance)));

        // Thread containing UDP sockets communicating with the roboRIO
        let udp_state = state.clone();
        let udp_rx = rx.clone();
        let udp_tx = tx.clone();
        thread::spawn(move || {
            udp_thread(udp_state, udp_tx, udp_rx, team_number);
        });

        let tcp_state = state.clone();
        thread::spawn(move || {
            tcp_thread(tcp_state, rx, team_number);
        });

        DriverStation {
            thread_comm: tx,
            state,
        }
    }

    /// Provides a closure that will be called when constructing outbound packets to append joystick values
    pub fn set_joystick_supplier(&mut self, supplier: impl Fn() -> Vec<JoystickValue> + Send + Sync + 'static) {
        self.state.lock().unwrap().set_joystick_supplier(supplier);
    }

    pub fn set_tcp_consumer(&mut self, consumer: impl Fn(TcpPacket) + Send + Sync + 'static) {
        self.state.lock().unwrap().set_tcp_consumer(consumer);
    }

    /// Changes the alliance for the given `DriverStation`
    pub fn set_alliance(&mut self, alliance: Alliance) {
        self.state.lock().unwrap().set_alliance(alliance);
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.state.lock().unwrap().set_mode(mode);
    }

    pub fn mode(&self) -> Mode {
        *self.state.lock().unwrap().mode()
    }

    /// Enables outputs on the robot
    pub fn enable(&mut self) {
        self.state.lock().unwrap().enable();
    }

    pub fn restart_code(&mut self) {
        self.state.lock().unwrap().request(Request::RESTART_CODE);
    }

    pub fn enabled(&self) -> bool {
        *self.state.lock().unwrap().enabled()
    }

    pub fn trace(&self) -> Trace {
        self.state.lock().unwrap().trace().clone()
    }

    pub fn battery_voltage(&self) -> f32 {
        *self.state.lock().unwrap().battery_voltage()
    }

    /// Disables outputs on the robot and disallows enabling it until the code is restarted.
    pub fn estop(&mut self) {
        self.state.lock().unwrap().estop();
    }

    /// Disables outputs on the robot
    pub fn disable(&mut self) {
        self.state.lock().unwrap().disable()
    }
}


impl Drop for DriverStation {
    fn drop(&mut self) {
        // When this struct is dropped the threads that we spawned should be stopped otherwise we're leaking
        self.thread_comm.send(Signal::Disconnect).unwrap();
    }
}

pub enum Signal {
    Disconnect,
    ConnectTcp,
}