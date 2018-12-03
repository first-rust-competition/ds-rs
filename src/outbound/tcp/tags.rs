use super::JoystickType;

use byteorder::{WriteBytesExt, BigEndian};

pub enum TcpTag {
    MatchInfo(MatchInfo),
    GameData(GameData),
}

pub trait OutgoingTcpTag {
    fn id(&self) -> u8;

    fn data(&self) -> Vec<u8>;

    fn construct(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(self.id());
        buf.extend(self.data());

        let mut out = Vec::new();
        out.write_u16::<BigEndian>(buf.len() as u16);
        out.extend(buf);

        out
    }
}

pub struct JoystickDescriptor {
    index: u8,
    is_xbox: bool,
    joystick_type: JoystickType,
    name: String,
    
    //TODO: finish later im tired :screm:
}

pub struct MatchInfo {
    competition: String,
    match_type: MatchType
}

impl OutgoingTcpTag for MatchInfo {
    fn id(&self) -> u8 {
        0x07
    }

    fn data(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(self.competition.len() as u8);
        buf.extend_from_slice(self.competition.as_bytes());
        buf.push(self.match_type as u8);

        buf
    }
}

pub struct GameData {
    gsm: String,
}

impl OutgoingTcpTag for GameData {
    fn id(&self) -> u8 {
        0x0e
    }

    fn data(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(self.gsm.as_bytes());

        buf
    }
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum MatchType {
    None = 0,
    Practice = 1,
    Qualifications = 2,
    Eliminations = 3,
}
