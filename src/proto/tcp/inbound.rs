use crate::ext::BufExt;
use crate::Result as CResult;
use byteorder::{BigEndian, ReadBytesExt};
use bytes::{Buf, BytesMut};
use std::io::Error;
use std::{io, str};

/// Enum containing possible incoming TCP packets from the roboRIO
pub enum TcpPacket {
    /// Contains a message from the robot code's standard output
    Stdout(Stdout),
    Dummy,
}

pub(crate) trait IncomingTcpPacket: Sized {
    fn decode(buf: &mut impl Buf) -> CResult<Self>;
}

/// Contains data outputted to standard output from robot code. Can be consumed by API users to
/// display code logs
pub struct Stdout {
    pub timestamp: f32,
    pub message: String,
    pub seqnum: u16,
}

impl IncomingTcpPacket for Stdout {
    fn decode(buf: &mut impl Buf) -> CResult<Self> {
        let timestamp = buf.read_f32_be()?;
        let seqnum = buf.read_u16_be()?;
        let message = str::from_utf8(buf.bytes())?;
        Ok(Stdout {
            timestamp,
            message: message.to_string(),
            seqnum,
        })
    }
}
