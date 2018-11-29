//TODO: tags
use byteorder::{WriteBytesExt, BigEndian};

use crate::util::to_u8_vec;

pub trait Tag {
    fn id(&self) -> usize;

    fn data(&self) -> Vec<u8>;

    fn construct(&self) -> Vec<u8> {
        let mut buf = vec![self.id() as u8];
        buf.extend(self.data());

        buf.insert(0, buf.len() as u8);

        buf
    }
}

pub struct Countdown {
    seconds_remaining: f32,
}

impl Tag for Countdown {
    fn id(&self) -> usize {
        0x07
    }

    fn data(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(2);
        buf.write_f32::<BigEndian>(self.seconds_remaining);

        buf
    }
}

pub struct Joysticks {
    axes: Vec<i8>,
    buttons: Vec<bool>,
    povs: Vec<i16>,
}

impl Tag for Joysticks {
    fn id(&self) -> usize {
        0x0c
    }

    fn data(&self) -> Vec<u8> {
        let mut buf = vec![];
        buf.write_u8(self.axes.len() as u8);
        for axis in &self.axes {
            buf.write_i8(*axis);
        }

        let buttons = to_u8_vec(&self.buttons);
        buf.push(buttons.len() as u8);
        buf.extend(buttons);

        buf.push(self.povs.len() as u8);

        for pov in &self.povs {
            buf.write_i16::<BigEndian>(*pov);
        }

        buf
    }
}

//TODO: timezone tags

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_format() {
        let countdown = Countdown { seconds_remaining: 2f32 };
        let buf = countdown.construct();

        assert_eq!(buf, &[0x05, 0x07, 0x040, 0x0, 0x0, 0x0]);
    }

    #[test]
    fn verify_joysticks() {
        let joysticks = Joysticks {
            axes: vec![],
            buttons: vec![true, true, false, false, false, true, false],
            povs: vec![]
        };
        let buf = joysticks.construct();
        println!("{:?}", buf);
    }
}

