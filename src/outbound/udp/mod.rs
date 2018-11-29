pub mod types;

use byteorder::{WriteBytesExt, BigEndian};
use self::types::*;
use self::types::tags::*;

pub struct UdpControlPacket {
    pub seqnum: u16,
    pub control: Control,
    pub request: Option<Box<Request>>,
    pub alliance: Alliance,
    pub tags: Vec<Box<Tag>>,
}

impl UdpControlPacket {
    pub fn encode(&mut self) -> Vec<u8> {
        let mut buf = vec![];
        buf.write_u16::<BigEndian>(self.seqnum);
        buf.push(0x01); // comm version
        buf.push(self.control.bits());
        match &self.request {
            Some(ref req) => buf.push(req.code()),
            None => buf.push(0)
        }

        buf.push(self.alliance.0);


        for tag in &self.tags {
            buf.extend(tag.construct());
        }

        self.seqnum += 1;

        buf
    }
}