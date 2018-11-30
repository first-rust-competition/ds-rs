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

use gilrs::*;

pub struct DriverStation {
    thread_comm: Sender<Signal>,
    state: Arc<Mutex<State>>,
}

impl DriverStation {
    pub fn new(alliance: Alliance) -> DriverStation {
        let (tx, rx) = unbounded::<Signal>();
        let state = Arc::new(Mutex::new(State::new(alliance)));

        let joystick_state = state.clone();
        let js_rx = rx.clone();
        thread::spawn(move || {
            let mut gilrs = Gilrs::new().unwrap();
            let rx = js_rx;

            loop {
                match rx.try_recv() {
                    Ok(Signal::Disconnect) | Err(crossbeam_channel::TryRecvError::Disconnected) => break,
                    _ => {},
                }
                if let Some(ev) = gilrs.next_event() {
                    let mut state = joystick_state.lock().unwrap();
                    match ev.event {
                        EventType::AxisChanged(_, value, id) => state.joystick_update(JoystickValue::Axis { id: id.into_u32() as u8, value }),
                        EventType::ButtonChanged(_, value, id) => state.joystick_update(JoystickValue::Button { id: id.into_u32() as u8, pressed: value == 1.0 }),
                        _ => {},
                    }
                }
                thread::sleep(Duration::from_millis(20));
            }
        });

        let thread_state = state.clone();
        thread::spawn(move || {
            let mut last = Instant::now();
//            let udp_tx = UdpSocket::bind("10.40.69.1:5678").unwrap();
            let udp_tx = UdpSocket::bind("10.40.69.65:5678").unwrap();
            udp_tx.connect("10.40.69.2:1110").unwrap();

//            let udp_rx = UdpSocket::bind("10.40.69.1:1150").unwrap();
            let udp_rx = UdpSocket::bind("10.40.69.65:1150").unwrap();
            udp_rx.set_nonblocking(true).unwrap();

            println!("UDP sockets open.");

            loop {
                match rx.try_recv() {
                    Ok(Signal::Disconnect) | Err(crossbeam_channel::TryRecvError::Disconnected) => break,
                    _ => {}
                }

                let mut buf = [0u8; 100];

                match udp_rx.recv_from(&mut buf[..]) {
                    Ok(_) => {
                        let mut state = thread_state.lock().unwrap();
                        if let Ok(packet) = UdpResponsePacket::decode(&buf[..], state.seqnum()) {
                            println!("Received packet {:?}", packet);
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

                if last.elapsed() >= Duration::from_millis(20) {
                    last = Instant::now();
                    let mut state = thread_state.lock().unwrap();
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
        self.thread_comm.send(Signal::Disconnect).unwrap();
    }
}

pub enum Signal {
    Disconnect,
}