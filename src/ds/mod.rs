use crossbeam_channel::{self, Sender, unbounded};

use std::thread;

pub mod state;
mod conn;

use self::state::*;
use self::conn::*;

use std::sync::{Arc, Mutex};

use crate::outbound::udp::types::Alliance;

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

    /// Enables outputs on the robot
    pub fn enable(&mut self) {
        self.state.lock().unwrap().enable();
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