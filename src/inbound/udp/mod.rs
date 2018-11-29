pub mod types;

use self::types::*;
use self::types::tag::*;

use crate::Result;
use failure::bail;

use byteorder::{ReadBytesExt, BigEndian};

#[derive(Debug)]
pub struct UdpResponsePacket {
    seqnum: u16,
    status: Status,
    trace: Trace,
    battery: f32,
    first_req: bool,
    tags: Vec<Box<Tag>>
}

impl UdpResponsePacket {
    pub fn decode(mut buf: &[u8], expected_seqnum: u16) -> Result<UdpResponsePacket> {
        let seqnum = buf.read_u16::<BigEndian>()?;
        if seqnum != expected_seqnum {
            bail!("Unexpeced sequence number {}", seqnum);
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
            first_req,
            tags: Vec::new(),
        })
    }
}