use byteorder::{ReadBytesExt, BigEndian};
use crate::Result;
use std::str;

pub enum TcpPacket {
    Stdout(Stdout),
    ErrorMessage(ErrorMessage),
}

pub trait IncomingTcpPacket: Sized {
    fn decode(buf: &[u8]) -> Result<Self>;
}

/// Contains data outputted to standard output from robot code. Can be consumed by API users to
/// display code logs
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

pub struct ErrorMessage {
    pub timestamp: f32,
    pub seqnum: u16,
    pub error_code: u16,
    pub is_error: bool,
    pub details: String,
    pub location: String,
    pub call_stack: String,
}

impl IncomingTcpPacket for ErrorMessage {
    fn decode(mut buf: &[u8]) -> Result<Self> {
        let timestamp = buf.read_f32::<BigEndian>()?;
        let seqnum = buf.read_u16::<BigEndian>()?;
        let _ = buf.read_u8()?;
        let error_code = buf.read_u16::<BigEndian>()?;
        let is_error = buf.read_u8()? == 1;
        let details = {
            let len = buf.read_u16::<BigEndian>()? as usize;
            str::from_utf8(&buf[..len])?
        };
        let location = {
            let len = buf.read_u16::<BigEndian>()? as usize;
            str::from_utf8(&buf[..len])?
        };
        let call_stack = {
            let len = buf.read_u16::<BigEndian>()? as usize;
            str::from_utf8(&buf[..len])?
        };

        Ok(ErrorMessage {
            timestamp,
            seqnum,
            error_code,
            is_error,
            details: details.to_string(),
            location: location.to_string(),
            call_stack: call_stack.to_string(),
        })
    }
}