use super::state::State;
use super::Signal;

use crate::util::*;
use crate::inbound::udp::UdpResponsePacket;
use crate::inbound::tcp::*;
use crate::outbound::udp::types::tags::{*, DateTime as DTTag};
use crate::outbound::tcp::*;
use crate::inbound::udp::types::Trace;

use std::net::{UdpSocket, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use std::io::{self, Write, Read, ErrorKind};
use std::thread;

use chrono::prelude::*;
use crossbeam_channel::{self, Receiver, Sender};
use byteorder::{ReadBytesExt, BigEndian};
use smallvec::SmallVec;

use failure::bail;

use crate::Result;

/// Contains the logic for sending and receiving messages over UDP to/from the roboRIO
pub(crate) fn udp_thread(state: Arc<Mutex<State>>, tx: Sender<Signal>, rx: Receiver<Signal>, target_ip: String) -> Result<()> {
    let mut tcp_connected = false;

    let mut last = Instant::now();

    let udp_tx = UdpSocket::bind("0.0.0.0:5678")?;
    udp_tx.connect(&format!("{}:1110", target_ip))?;

    let udp_rx = UdpSocket::bind("0.0.0.0:1150")?;
    udp_rx.set_nonblocking(true)?;

    // When we initially instruct the robot to estop, the inbound UDP packets
    // may not reflect the change immediately, and estop state is updated based on what the roboRIO says.
    // Toggled when estop is first instructed by the DS and remains for 3 loop iterations to allow the roboRIO to get the memo
    let mut estop_grace = false;
    let mut iterations = 0;

    loop {
        match rx.try_recv() {
            Ok(Signal::Disconnect) | Err(crossbeam_channel::TryRecvError::Disconnected) => break,
            Ok(Signal::Heartbeat) => tx.try_send(Signal::Heartbeat).unwrap(),
            _ => {}
        }

        // Buffer to hold the upcoming packet from the roborio
        let mut buf = [0u8; 100];

        match udp_rx.recv_from(&mut buf[..]) {
            Ok(_) => {
                let mut state = state.lock().unwrap();
                if let Ok(packet) = UdpResponsePacket::decode(&buf[..], state.seqnum()) {

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
                        state.queue_udp(UdpTag::DateTime(tag));

                        // hardcode the timezone because :screm:
                        // FIXME: maybe dont
                        let tz = Timezone::new("Canada/Eastern");
                        state.queue_udp(UdpTag::Timezone(tz));
                    }

                    if *state.estopped() && !packet.status.emergency_stopped() {
                        estop_grace = true;
                        iterations = 0;
                    }

                    if !estop_grace {
                        state.set_estop(packet.status.emergency_stopped());
                    }

                    if !packet.trace.is_code_started() {
                        state.set_estop(false); // Estop gets reset when code is deployed
                    }

                    // Update the state for the next iteration
                    state.increment_seqnum();
                    state.set_trace(packet.trace);
                    state.set_battery_voltage(packet.battery);
                    if !tcp_connected {
                        tcp_connected = true;
                        tx.try_send(Signal::ConnectTcp).unwrap();
                    }
                }
            }
            Err(e) => {
                // According to jes, if the LV DS doesn't get a response after 500ms it assumes the roborio is gone
                if last.elapsed() >= Duration::from_millis(500) {
                    bail!("Connection timed out.");
                }

                if e.kind() != io::ErrorKind::WouldBlock {
                    return Err(e.into());
                }
            }
        }

        // roboRIO packets should be >=20ms apart, once there should send control packet
        if last.elapsed() >= Duration::from_millis(20) {
            let mut state = state.lock().unwrap();
            last = Instant::now();
            udp_tx.send(&state.control().encode()[..])?;
        }

        // After 5 iterations the RIO's control packet should reflect the estop change
        if iterations >= 5 {
            estop_grace = false;
        }

        if estop_grace {
            iterations += 1;
        }
        thread::sleep(Duration::from_millis(20));
    }

    // This loop is exited only when we're told to disconnect. Clear any roboRIO state
    let mut state = state.lock().unwrap();
    state.set_trace(Trace::empty());

    Ok(())
}

/// Contains logic for communication to/from the roboRIO over TCP
pub(crate) fn tcp_thread(state: Arc<Mutex<State>>, rx: Receiver<Signal>, target_ip: String) -> Result<()> {
    match rx.recv() {
        Ok(Signal::Disconnect) | Err(_) => return Ok(()),
        _ => {}
    }

    let mut conn = TcpStream::connect(&format!("{}:1740", target_ip))?;
    // Because I can't split `conn` and set reads to nonblocking, settling for a timeout instead
    // Can be fairly long because TCP packets aren't very frequent, other than stdout
    conn.set_read_timeout(Some(Duration::from_secs(2)))?;

//    println!("TCP socket open.");

    loop {
        match rx.try_recv() {
            Ok(Signal::Disconnect) | Err(crossbeam_channel::TryRecvError::Disconnected) => break,
            _ => {}
        }

        // Nested scope because otherwise we could deadlock on `state` if TCP doesn't get anything before the next UDP packet needs to be sent
        {
            let mut state = state.lock().unwrap();
            if !state.pending_tcp().is_empty() {
                for tag in state.pending_tcp() {
                    match tag {
                        TcpTag::GameData(gd) => conn.write(&gd.construct()[..])?,
                        TcpTag::MatchInfo(mi) => conn.write(&mi.construct()[..])?,
                    };
                }
                state.pending_tcp_mut().clear();
            }
        }


        // Try to read a u16 which holds the size of an incoming packet.
        // At this point all we know is the u16, so no need to make the buffer bigger (but not big enough)
        // and potentially miss data
        let mut prelim = [0; 2];
        match conn.read(&mut prelim) {
            Ok(_) => {
                // prelim will hold the size of the incoming packet at this point
                let mut prelim = &prelim[..];
                let size = prelim.read_u16::<BigEndian>().unwrap();

                // At this point buf will hold the entire packet minus length prefix.
                let mut buf: SmallVec<[u8; 0x8000]> = smallvec![0u8; size as usize];
                conn.read_exact(&mut buf[..])?;

                let mut state = state.lock().unwrap();
                if let Some(ref mut consumer) = &mut state.tcp_consumer {
                    match buf.get(0) {
                        // stdout
                        Some(0x0c) => match Stdout::decode(&buf[1..]) {
                            Ok(stdout) => consumer(TcpPacket::Stdout(stdout)),
                            Err(e) => println!("ERROR DECODING STDOUT\n----\n{}", e),
                        }
                        //FIXME: Error message decoding is buggy
//                    Some(0x0b) => match ErrorMessage::decode(&buf[1..]) {
//                        Ok(err) => consumer(TcpPacket::ErrorMessage(err)),
//                        Err(e) => println!("ERROR DECODING ERROR MESSAGE\n----\n{}", e),
//                    }

                        None => {
                            // Something has gone terrible terribly wrong, but i dont want to panic so its a thonk
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => if e.kind() != ErrorKind::WouldBlock {
                return Err(e.into());
            }
        }
    }

    Ok(())
}
