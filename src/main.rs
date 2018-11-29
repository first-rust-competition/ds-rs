#![feature(nll)]

extern crate tokio;
extern crate futures;
#[macro_use]
extern crate bitflags;
extern crate byteorder;
extern crate failure;

mod outbound;
mod inbound;
pub mod util;

use std::net::UdpSocket;
use std::thread;

use self::outbound::udp::UdpControlPacket;
use self::outbound::udp::types::*;

use self::inbound::udp::UdpResponsePacket;

pub type Result<T> = std::result::Result<T, failure::Error>;

fn main() {
    let mut sock_recv = UdpSocket::bind("10.40.69.1:1150").unwrap();
    let mut sock_send = UdpSocket::bind("10.40.69.1:5678").unwrap();
    sock_send.connect("10.40.69.2:1110").unwrap();

    let mut control = UdpControlPacket {
        seqnum: 1,
        control: Control::empty(),
        request: None,
        alliance: Alliance::new_red(1),
        tags: Vec::new()
    };

    println!("Connection established.");

    println!("Looping");
    loop {
        let seqnum = control.seqnum;
        sock_send.send(&control.encode()[..]).unwrap();
        let mut buf = [0; 20];
        sock_recv.recv_from(&mut buf[..]).unwrap();
        let res = UdpResponsePacket::decode(&buf[..], seqnum).unwrap();
        println!("{:#?}", res);
        thread::sleep_ms(20);
    }
}
