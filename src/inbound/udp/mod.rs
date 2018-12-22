pub mod types;

use self::types::*;

use crate::Result;
use failure::bail;

use byteorder::{ReadBytesExt, BigEndian};

/// Response packet sent by the RIO over UDP every ~20ms.
#[derive(Debug)]
pub struct UdpResponsePacket {
    pub seqnum: u16,
    pub status: Status,
    pub trace: Trace,
    pub battery: f32,
    pub need_date: bool,
}

impl UdpResponsePacket {
    /// Attempts to decode a valid response packet from the given buffer
    /// Will return Err() if any of the reads fail, or if the sequence number of the packet
    /// doesn't match the expected sequence number
    pub fn decode(mut buf: &[u8], expected_seqnum: u16) -> Result<UdpResponsePacket> {
        let seqnum = buf.read_u16::<BigEndian>()?;
        if seqnum != expected_seqnum {
            bail!("Unexpeced sequence number {}. Expected {}", seqnum, expected_seqnum);
        }

        buf.read_u8()?; // Get rid of comm version

        let status = Status::from_bits(buf.read_u8()?).unwrap();
        let trace = Trace::from_bits(buf.read_u8()?).unwrap();

        let battery = {
            let high = buf.read_u8()?;
            let low = buf.read_u8()?;
            f32::from(high) + f32::from(low) / 256f32
        };

        let need_date = buf.read_u8()? == 1;

        Ok(UdpResponsePacket {
            seqnum,
            status,
            trace,
            battery,
            need_date,
        })
    }
}
