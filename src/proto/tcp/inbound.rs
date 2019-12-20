use crate::ext::BufExt;
use crate::Result as CResult;
use bytes::Buf;
use std::io::{Error, ErrorKind};
use std::str;

/// Enum containing possible incoming TCP packets from the roboRIO
#[derive(Debug)]
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
#[derive(Debug)]
pub struct Stdout {
    pub timestamp: f32,
    pub message: String,
    pub seqnum: u16,
}


impl Stdout {
    pub fn decode(buf: &mut impl Buf, len: usize) -> CResult<Self> {
        let timestamp = buf.read_f32_be()?;
        let seqnum = buf.read_u16_be()?;
        let mut v = vec![0; len - 6];
        if buf.remaining() < v.len() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Not enough data").into());
        }
        buf.copy_to_slice(&mut v[..]);
        let message = str::from_utf8(&v[..])?;
        Ok(Stdout {
            timestamp,
            message: message.to_string(),
            seqnum,
        })
    }
}
