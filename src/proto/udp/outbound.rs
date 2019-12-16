pub mod types;

use self::types::tags::*;
use self::types::*;
use crate::proto::udp::inbound::UdpResponsePacket;
use byteorder::{BigEndian, WriteBytesExt};
use bytes::BytesMut;
use std::io;
use std::io::Error;

/// UDP control packet to send to the roboRIO
pub struct UdpControlPacket {
    pub(crate) seqnum: u16,
    pub(crate) control: Control,
    pub(crate) request: Option<Request>,
    pub(crate) alliance: Alliance,
    pub(crate) tags: Vec<Box<dyn Tag>>,
}

impl UdpControlPacket {
    /// Encodes the current state of the packet into a vec to send to the roboRIO
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = vec![];
        buf.write_u16::<BigEndian>(self.seqnum).unwrap();
        buf.push(0x01); // comm version
        buf.push(self.control.bits());
        match &self.request {
            Some(ref req) => buf.push(req.bits()),
            None => buf.push(0),
        }

        buf.push(self.alliance.0);

        for tag in &self.tags {
            buf.extend(tag.construct());
        }

        buf
    }
}
