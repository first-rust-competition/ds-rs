//TODO: Incoming tags, atm im not really bothering

use crate::Result;
use byteorder::{BigEndian, ReadBytesExt};

pub trait Tag: Sized {
    fn decode(buf: &[u8]) -> Result<Self>;
}

#[derive(Debug)]
pub enum TagType {

}

pub struct JoystickOutput {
    output: u32,
    left: u16,
    right: u16
}

impl Tag for JoystickOutput {
    fn decode(mut buf: &[u8]) -> Result<Self> {
        let output = buf.read_u32::<BigEndian>()?;
        let left = buf.read_u16::<BigEndian>()?;
        let right = buf.read_u16::<BigEndian>()?;

        Ok(JoystickOutput {
            output,
            left,
            right
        })
    }
}

pub struct DiskInfo {
    free_space: u32
}

impl Tag for DiskInfo {
    fn decode(mut buf: &[u8]) -> Result<Self> {
        let free_space = buf.read_u32::<BigEndian> ()?;
        Ok(DiskInfo {
            free_space
        })
    }
}
