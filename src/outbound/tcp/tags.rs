use super::JoystickType;

pub trait Tag {
    fn id(&self) -> u8;

    fn data(&self) -> Vec<u8>;

    fn construct(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(self.id());
        buf.extend(self.data());
        buf.insert(0, buf.len() as u8);

        buf
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

impl Tag for MatchInfo {
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

impl Tag for GameData {
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
