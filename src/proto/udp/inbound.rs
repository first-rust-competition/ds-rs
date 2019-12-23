pub mod types;

use self::types::*;

use crate::ext::BufExt;
use crate::Result;

use bytes::Buf;

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
    /// Will return Err() if any of the reads fail.
    pub fn decode(buf: &mut impl Buf) -> Result<(UdpResponsePacket, usize)> {
        let mut len = 0;
        let seqnum = buf.read_u16_be()?;
        len += 2;

        buf.read_u8()?; // Get rid of comm version
        len += 1;

        let status = Status::from_bits(buf.read_u8()?).unwrap();
        let trace = Trace::from_bits(buf.read_u8()?).unwrap();
        len += 2;

        let battery = {
            let high = buf.read_u8()?;
            let low = buf.read_u8()?;
            f32::from(high) + f32::from(low) / 256f32
        };
        len += 2;

        let need_date = buf.read_u8()? == 1;
        len += 1;

        use crate::util::InboundTag;
        while let Ok(tag_id) = buf.read_u8() {
            len += 1;
            match tag_id {
                0x01 => {
                    types::tags::JoystickOutput::chomp(buf)?;
                    len += 8;
                }
                0x04 => {
                    types::tags::DiskInfo::chomp(buf)?;
                    len += 4;
                }
                0x05 => {
                    types::tags::CPUInfo::chomp(buf)?;
                    len += 20;
                }
                0x06 => {
                    types::tags::RAMInfo::chomp(buf)?;
                    len += 8;
                }
                0x08 => {
                    types::tags::PDPLog::chomp(buf)?;
                    len += 25;
                }
                0x09 => {
                    types::tags::Unknown::chomp(buf)?;
                    len += 9;
                }
                0x0e => {
                    types::tags::CANMetrics::chomp(buf)?;
                    len += 14;
                }
                _ => {}
            }
        }

        Ok((
            UdpResponsePacket {
                seqnum,
                status,
                trace,
                battery,
                need_date,
            },
            len,
        ))
    }
}
