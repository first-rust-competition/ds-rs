use byteorder::{ReadBytesExt, BigEndian};
use crate::Result;
use std::str;

pub enum TcpPacket {
    Stdout(Stdout),
}

pub trait IncomingTcpPacket: Sized {
    fn decode(buf: &[u8]) -> Result<Self>;
}

pub struct Stdout {
    pub timestamp: f32,
    pub message: String,
    pub seqnum: u16
}

impl IncomingTcpPacket for Stdout {
    fn decode(mut buf: &[u8]) -> Result<Self> {
        let timestamp = buf.read_f32::<BigEndian>()?;
        let seqnum = buf.read_u16::<BigEndian>()?;
        let message = str::from_utf8(buf)?;
        Ok(Stdout {
            timestamp,
            message: message.to_string(),
            seqnum
        })
    }
}