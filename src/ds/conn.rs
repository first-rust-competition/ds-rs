use super::Signal;

use crate::proto::udp::inbound::types::Trace;
use crate::proto::udp::inbound::UdpResponsePacket;
use crate::proto::udp::outbound::types::tags::{DateTime as DTTag, *};

use futures_channel::mpsc::unbounded;
use futures_channel::mpsc::UnboundedReceiver;
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpStream, UdpSocket};
use tokio::time;
use tokio_util::codec::Decoder;
use tokio_util::udp::UdpFramed;

use chrono::prelude::*;

use crate::proto::tcp::DsTcpCodec;
use crate::proto::udp::DsUdpCodec;
use crate::Result;

use crate::ds::state::DsState;
use crate::proto::tcp::outbound::TcpTag;
use futures_util::future::Either;
use futures_util::stream::select;

/// The root task of the tokio runtime.
///
/// This task connects to the receiving UDP port, and spawns tasks for UDP sending, and for TCP communications once the connection to the RIO has been established.
pub(crate) async fn udp_conn(
    state: Arc<DsState>,
    mut target_ip: String,
    rx: UnboundedReceiver<Signal>,
) -> Result<()> {
    let mut tcp_connected = false;
    let mut tcp_tx = None;

    let udp_rx = UdpSocket::bind("0.0.0.0:1150").await?;
    let udp_rx = UdpFramed::new(udp_rx, DsUdpCodec);

    let (fwd_tx, fwd_rx) = unbounded::<Signal>();

    let send_state = state.clone();
    let target = target_ip.clone();
    tokio::spawn(async move {
        let mut udp_tx = UdpSocket::bind("0.0.0.0:0")
            .await
            .expect("Failed to bind tx socket");
        udp_tx
            .connect(&format!("{}:1110", target))
            .await
            .expect("Failed to connect to target");

        let interval = time::interval(Duration::from_millis(20));

        let mut stream = select(interval.map(Either::Left), fwd_rx.map(Either::Right));

        loop {
            let item = stream.next().await.unwrap();
            match item {
                Either::Left(_) => {
                    let mut state = send_state.send().lock().await;
                    let v = state.control().encode();
                    udp_tx.send(&v[..]).await.expect("Failed to send packet");
                    state.increment_seqnum();
                }
                Either::Right(sig) => match sig {
                    Signal::NewTarget(ip) => {
                        let mut state = send_state.send().lock().await;
                        state.reset_seqnum();
                        state.disable();
                        send_state.recv().lock().await.set_trace(Trace::empty());
                        udp_tx
                            .connect(&format!("{}:1110", &ip))
                            .await
                            .expect("Failed to connect to new target");
                    }
                    _ => {}
                },
            }
        }
    });

    let mut stream = select(udp_rx.map(Either::Left), rx.map(Either::Right));

    while let Some(item) = stream.next().await {
        match item {
            Either::Left(packet) => match packet {
                Ok(packet) => {
                    let (packet, _): (UdpResponsePacket, _) = packet;
                    let mut _state = state.recv().lock().await;

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
                        state.send().lock().await.queue_udp(UdpTag::DateTime(tag));
                    }

                    if !tcp_connected {
                        let (tx, rx) = unbounded::<Signal>();
                        tcp_tx = Some(tx);
                        tokio::spawn(tcp_conn(state.clone(), target_ip.clone(), rx));
                        tcp_connected = true;
                    }

                    if packet.status.emergency_stopped() {
                        let mut send = state.send().lock().await;
                        if !send.estopped() {
                            send.estop();
                        }
                    }

                    _state.set_trace(packet.trace);
                    _state.set_battery_voltage(packet.battery);
                }
                Err(e) => println!("Error decoding packet: {:?}", e),
            },
            Either::Right(sig) => match sig {
                Signal::Disconnect => return Ok(()),
                Signal::NewTarget(ref target) => {
                    if let Some(ref tcp_tx) = tcp_tx {
                        let _ = tcp_tx.unbounded_send(Signal::Disconnect);
                        tcp_connected = false;
                    }

                    target_ip = target.clone();

                    fwd_tx.unbounded_send(sig)?;
                }
            },
        }
    }
    Ok(())
}

/// tokio task for all TCP communications
///
/// This task will decode incoming TCP packets, and call the tcp consumer defined in `state` if it exists.
/// It will also accept packets to send from a channel set in `state`, for tasks such as defining game data.
pub(crate) async fn tcp_conn(
    state: Arc<DsState>,
    target_ip: String,
    rx: UnboundedReceiver<Signal>,
) -> Result<()> {
    let conn = TcpStream::connect(&format!("{}:1740", target_ip)).await?;
    let codec = DsTcpCodec.framed(conn);
    let (mut codec_tx, codec_rx) = codec.split();

    let (tag_tx, tag_rx) = unbounded::<TcpTag>();
    state.tcp().lock().await.set_tcp_tx(Some(tag_tx));

    let stream = select(codec_rx.map(Either::Left), rx.map(Either::Right));
    let mut stream = select(stream.map(Either::Left), tag_rx.map(Either::Right));

    let state = state.tcp();
    while let Some(msg) = stream.next().await {
        match msg {
            Either::Left(left) => match left {
                Either::Left(packet) => {
                    if let Ok(packet) = packet {
                        let mut state = state.lock().await;
                        if let Some(ref mut consumer) = state.tcp_consumer {
                            consumer(packet);
                        }
                    }
                }
                Either::Right(_) => {
                    state.lock().await.set_tcp_tx(None);
                }
            },
            Either::Right(tag) => {
                let _ = codec_tx.send(tag).await;
            }
        }
    }
    Ok(())
}
