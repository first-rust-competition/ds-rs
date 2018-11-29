pub mod types;

use self::types::*;
use self::types::tag::*;

use crate::Result;
use failure::bail;

use byteorder::{ReadBytesExt, BigEndian};

#[derive(Debug)]
pub struct UdpResponsePacket {
    pub seqnum: u16,
    pub status: Status,
    pub trace: Trace,
    pub battery: f32,
    pub need_date: bool,
    pub tags: Vec<TagType>,
}

impl UdpResponsePacket {
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
            (high as f32) + (low as f32) / 256f32
        };

        let first_req = buf.read_u8()? == 1;
        // ignore tags for now

        Ok(UdpResponsePacket {
            seqnum,
            status,
            trace,
            battery,
            need_date: first_req,
            tags: Vec::new(),
        })
    }
}