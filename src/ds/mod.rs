use crossbeam_channel::{self, Sender, unbounded};

use std::thread;
use std::io;

pub mod state;
mod conn;

use self::state::*;
use self::conn::*;

use chrono::prelude::*;

use std::net::UdpSocket;
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex};

use crate::inbound::udp::UdpResponsePacket;
use crate::outbound::udp::UdpControlPacket;
use crate::outbound::udp::types::Alliance;
use crate::outbound::udp::types::tags::{*, DateTime as DTTag};
use crate::util::*;

use gilrs::*;

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

        // Thread containing logic to read from joysticks connected to the computer, and push them to the state to update the roborio
        let joystick_state = state.clone();
        let js_rx = rx.clone();
        thread::spawn(move || {
            let mut gilrs = Gilrs::new().unwrap();
            let rx = js_rx;

            loop {
                // If we receive Disconnect from the channel we should break out of the loop
                match rx.try_recv() {
                    Ok(Signal::Disconnect) | Err(crossbeam_channel::TryRecvError::Disconnected) => break,
                    _ => {},
                }
                if let Some(ev) = gilrs.next_event() {
                    let mut state = joystick_state.lock().unwrap();
                    match ev.event {
                        EventType::AxisChanged(axis, value, _) => {
                            state.joystick_update(JoystickValue::Axis { id: axis_to_roborio(axis), value });
                        },
                        EventType::ButtonChanged(button, value, _) => {
                            let roborio_id = button_to_roborio(button);

                            if button == Button::LeftTrigger2 || button == Button::RightTrigger2 {
                                state.joystick_update(JoystickValue::Axis { id: roborio_id, value });
                            }else {
                                state.joystick_update(JoystickValue::Button { id: roborio_id, pressed: value == 1.0 })
                            }
                        },
                        _ => {}
                    }
                }
                thread::sleep(Duration::from_millis(2));
            }
        });

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

    pub fn enable(&mut self) {
        self.state.lock().unwrap().enable();
    }

    pub fn estop(&mut self) {
        self.state.lock().unwrap().estop();
    }

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