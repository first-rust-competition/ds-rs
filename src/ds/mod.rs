use crossbeam_channel::{self, Sender, unbounded};

use std::thread;
use std::io;

pub mod state;

use self::state::*;

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
    /// Creates a new driver station for the given alliance station
    ///
    /// Will eventually take more information such as the roboRIO it should connect to
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
                            if button == Button::LeftTrigger2 || button == Button::RightTrigger2 {
                                state.joystick_update(JoystickValue::Axis { id: button_to_roborio(button), value });
                            }
                        },
                        _ => {}
                    }
                }
                thread::sleep(Duration::from_millis(20));
            }
        });


        // Thread containing UDP sockets communicating with the roboRIO
        let udp_state = state.clone();
        thread::spawn(move || {
            let target_ip = ip_from_team_number(team_number);
            let mut last = Instant::now();
//            let udp_tx = UdpSocket::bind("10.40.69.1:5678").unwrap();
            let udp_tx = UdpSocket::bind("10.40.69.65:5678").unwrap();
            udp_tx.connect(&format!("{}:1110", target_ip)).unwrap();

//            let udp_rx = UdpSocket::bind("10.40.69.1:1150").unwrap();
            let udp_rx = UdpSocket::bind("10.40.69.65:1150").unwrap();
            udp_rx.set_nonblocking(true).unwrap();

            println!("UDP sockets open.");

            loop {
                match rx.try_recv() {
                    Ok(Signal::Disconnect) | Err(crossbeam_channel::TryRecvError::Disconnected) => break,
                    _ => {}
                }

                // Buffer to hold the upcoming packet from the roborio
                let mut buf = [0u8; 100];

                match udp_rx.recv_from(&mut buf[..]) {
                    Ok(_) => {
                        let mut state = udp_state.lock().unwrap();
                        if let Ok(packet) = UdpResponsePacket::decode(&buf[..], state.seqnum()) {
//                            println!("Received packet {:?}", packet);

                            // if need_date is set, the roborio expects DateTime and Timezone tags on the following heartbeat
                            if packet.need_date {
                                let local = Utc::now();
                                let micros = local.naive_utc().timestamp_subsec_micros();
                                let second = local.time().second() as u8;
                                let minute = local.time().minute() as u8;
                                let hour = local.time().hour() as u8;
                                let day = local.date().day() as u8;
                                let month = local.date().month0() as u8;
                                let year = (local.date().year() - 1900) as u8;
                                let tag = DTTag::new(micros, second, minute, hour, day, month, year);
                                state.queue(TagType::DateTime(tag));

                                let tz = Timezone::new("Canada/Eastern");
                                state.queue(TagType::Timezone(tz));
                            }
                            // Update the state for the next iteration
                            let mode = Mode::from_status(packet.status).unwrap();
                            state.set_mode(mode);
                            state.increment_seqnum();
                        }
                    }
                    Err(e) => {
                        if e.kind() != io::ErrorKind::WouldBlock {
                            panic!("{}", e);
                        }
                    }
                }

                // roboRIO packets should be >=20ms apart, once there should send control packet
                if last.elapsed() >= Duration::from_millis(20) {
                    let mut state = udp_state.lock().unwrap();
                    last = Instant::now();
                    udp_tx.send(&state.control().encode()[..]).unwrap();
                }

                thread::sleep(Duration::from_millis(20));
            }
        });

        DriverStation {
            thread_comm: tx,
            state,
        }
    }

    pub fn enable(&mut self) {
        self.state.lock().unwrap().enable();
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
}