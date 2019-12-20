//! This module contains various tags that can be attached to the outbound UDP packet
//! The `Tag` trait contains the core logic, and is inherited by structs with specific roles

use byteorder::{BigEndian, WriteBytesExt};

use crate::util::to_u8_vec;

/// Enum wrapping possible outgoing UDP tags
#[derive(Clone, Debug)]
pub enum UdpTag {
    /// Tag sent to inform user code of the time left in the current mode
    Countdown(Countdown),
    /// Tag sent to provide joystick input to the user code
    Joysticks(Joysticks),
    /// Tag sent to update the roboRIO system clock to match that of the driver station
    DateTime(DateTime),
    /// Tag sent to update the roboRIO timezone. Sent alongside the DateTime tag
    Timezone(Timezone),
}

/// Represents an outgoing UDP tag
pub(crate) trait Tag: Send {
    fn id(&self) -> usize;

    fn data(&self) -> Vec<u8>;

    fn construct(&self) -> Vec<u8> {
        let mut buf = vec![self.id() as u8];
        buf.extend(self.data());

        buf.insert(0, buf.len() as u8);

        buf
    }
}

/// Tag containing the time remaining in the current mode
#[derive(Clone, Debug)]
pub struct Countdown {
    seconds_remaining: f32,
}

impl Countdown {
    pub fn new(seconds: f32) -> Countdown {
        Countdown {
            seconds_remaining: seconds,
        }
    }
}

impl Tag for Countdown {
    fn id(&self) -> usize {
        0x07
    }

    fn data(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(2);
        buf.write_f32::<BigEndian>(self.seconds_remaining).unwrap();

        buf
    }
}

/// Tag containing values from joysticks
#[derive(Clone, Debug)]
pub struct Joysticks {
    axes: Vec<i8>,
    buttons: Vec<bool>,
    povs: Vec<i16>,
}

impl Joysticks {
    pub fn new(axes: Vec<i8>, buttons: Vec<bool>, povs: Vec<i16>) -> Joysticks {
        Joysticks {
            axes,
            buttons,
            povs,
        }
    }
}

impl Tag for Joysticks {
    fn id(&self) -> usize {
        0x0c
    }

    fn data(&self) -> Vec<u8> {
        let mut buf = vec![];
        buf.write_u8(self.axes.len() as u8).unwrap();
        for axis in &self.axes {
            buf.write_i8(*axis).unwrap();
        }

        let buttons = to_u8_vec(&self.buttons);

        buf.push(10);
        buf.extend(buttons);

        buf.push(self.povs.len() as u8);

        for pov in &self.povs {
            buf.write_i16::<BigEndian>(*pov).unwrap();
        }

        buf
    }
}

/// Tag containing the current date and time in UTC
#[derive(Clone, Debug)]
pub struct DateTime {
    micros: u32,
    second: u8,
    minute: u8,
    hour: u8,
    day: u8,
    month: u8,
    year: u8,
}

impl DateTime {
    pub fn new(
        micros: u32,
        second: u8,
        minute: u8,
        hour: u8,
        day: u8,
        month: u8,
        year: u8,
    ) -> DateTime {
        DateTime {
            micros,
            second,
            minute,
            hour,
            day,
            month,
            year,
        }
    }
}

impl Tag for DateTime {
    fn id(&self) -> usize {
        0x0f
    }

    fn data(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.write_u32::<BigEndian>(self.micros).unwrap();
        buf.push(self.second);
        buf.push(self.minute);
        buf.push(self.hour);
        buf.push(self.day);
        buf.push(self.month);
        buf.push(self.year);

        buf
    }
}

/// Tag containing the current timezone of the RIO
#[derive(Clone, Debug)]
pub struct Timezone {
    tz: String,
}

impl Timezone {
    pub fn new(tz: &str) -> Timezone {
        Timezone { tz: tz.to_string() }
    }
}

impl Tag for Timezone {
    fn id(&self) -> usize {
        0x10
    }

    fn data(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(self.tz.as_bytes());

        buf
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_format() {
        let countdown = Countdown {
            seconds_remaining: 2f32,
        };
        let buf = countdown.construct();

        assert_eq!(buf, &[0x05, 0x07, 0x040, 0x0, 0x0, 0x0]);
    }

    #[test]
    fn verify_joysticks() {
        let joysticks = Joysticks {
            axes: vec![],
            buttons: vec![true, true, false, false, false, true, false],
            povs: vec![],
        };
        let buf = joysticks.construct();
        println!("{:?}", buf);
    }
}
