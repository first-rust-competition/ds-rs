use crossbeam_channel::{self, Sender, Receiver, unbounded};
use failure::bail;

use std::thread;

pub mod state;
mod conn;

use self::state::*;
use self::conn::*;

use std::sync::{Arc, Mutex};
use std::net::UdpSocket;

use crate::outbound::udp::types::{Request, Alliance};
use crate::outbound::tcp::tags::*;
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
            let monkas_tate = udp_state.clone();
            if udp_thread(udp_state, udp_tx, udp_rx, team_number).is_err() {
                let mut state = monkas_tate.lock().unwrap();
                state.set_trace(Trace::empty());
                state.set_battery_voltage(0.0);
            }
        });

        let tcp_state = state.clone();
        let tcp_rx = rx.clone();
        thread::spawn(move || {
            tcp_thread(tcp_state, tcp_rx, team_number);
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

        match self.thread_rx.recv() {
            Ok(Signal::Heartbeat) => Ok(true),
            Err(e) => Err(e.into()),
            _ => unreachable!()
        }
    }

    pub fn reconnect(&mut self) -> Result<()> {
//        self.thread_tx.try_send(Signal::Disconnect).unwrap();

        if self.state.is_poisoned() {
            // The alliance is rarely altered, so forcing a lock here to grab its value should be fine.
            // Poisoned contents are left alone and then dropped once we replace it.
            let alliance = {
                let state = self.state.lock().err().unwrap().into_inner();
                state.alliance
            };

            self.state = Arc::new(Mutex::new(State::new(alliance)));
        }

        let team_number = self.team_number;

        let udp_state = self.state.clone();
        let udp_rx = self.thread_rx.clone();
        let udp_tx = self.thread_tx.clone();
        thread::spawn(move || {
            let monkas_tate = udp_state.clone();
            if udp_thread(udp_state, udp_tx, udp_rx, team_number).is_err() {
                let mut state = monkas_tate.lock().unwrap();
                state.set_trace(Trace::empty());
                state.set_battery_voltage(0.0);
            }
        });

        let tcp_state = self.state.clone();
        let tcp_rx = self.thread_rx.clone();
        thread::spawn(move || {
            tcp_thread(tcp_state, tcp_rx, team_number);
        });

        Ok(())
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

    pub fn set_game_specific_message(&mut self, message: &str) -> Result<()> {
        if message.len() != 3 {
            bail!("Message should be 3 characters long");
        }

        self.state.lock().unwrap().queue_tcp(TcpTag::GameData(GameData { gsm: message.to_string() }));
        Ok(())
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

    pub fn restart_roborio(&mut self) {
        self.state.lock().unwrap().request(Request::REBOOT_ROBORIO);
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

    pub fn estopped(&self) -> bool {
        *self.state.lock().unwrap().estopped()
    }

    /// Disables outputs on the robot
    pub fn disable(&mut self) {
        self.state.lock().unwrap().disable()
    }
}


impl Drop for DriverStation {
    fn drop(&mut self) {
        // When this struct is dropped the threads that we spawned should be stopped otherwise we're leaking
        self.thread_tx.send(Signal::Disconnect).unwrap();
    }
}

pub enum Signal {
    Disconnect,
    ConnectTcp,
    Heartbeat,
}