use crate::inbound::udp::types::Status;
use crate::outbound::udp::UdpControlPacket;
use crate::outbound::udp::types::*;
use crate::outbound::udp::types::tags::*;


pub struct State {
    mode: Mode,
    udp_seqnum: u16,
    enabled: bool,
    alliance: Alliance,
    queued_tags: Vec<TagType>,
}

impl State {
    pub fn new(alliance: Alliance) -> State {
        State {
            mode: Mode::Teleoperated,
            udp_seqnum: 1,
            enabled: false,
            alliance,
            queued_tags: Vec::new()
        }
    }

    pub fn queue(&mut self, tag: TagType) {
        self.queued_tags.push(tag);
    }

    pub fn control(&mut self) -> UdpControlPacket {
        let mut control = self.mode.to_control();

        if self.enabled {
            control |= Control::ENABLED;
        }

        let mut tags: Vec<Box<Tag>> = Vec::new();

        for tag in self.queued_tags.clone() {
            match tag {
                TagType::Timezone(tz) => tags.push(Box::new(tz)),
                TagType::DateTime(dt) => tags.push(Box::new(dt)),
                TagType::Joysticks(joy) => tags.push(Box::new(joy)),
                TagType::Countdown(cnt) => tags.push(Box::new(cnt)),
            }
        }

        self.queued_tags.clear();

        UdpControlPacket {
            seqnum: self.udp_seqnum,
            control,
            request: None,
            alliance: self.alliance,
            tags
        }
    }

    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn increment_seqnum(&mut self) {
        self.udp_seqnum += 1;
    }

    pub fn seqnum(&self) -> u16 {
        self.udp_seqnum
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

pub enum Mode {
    Autonomous,
    Teleoperated,
    Test,
}

impl Mode {
    pub fn from_status(status: Status) -> Option<Mode> {
        if status & Status::TELEOP == Status::TELEOP {
            Some(Mode::Teleoperated)
        }else if status & Status::AUTO == Status::AUTO {
            Some(Mode::Autonomous)
        }else if status & Status::TEST == Status::TEST {
            Some(Mode::Test)
        }else {
            None
        }
    }

    fn to_control(&self) -> Control {
        match *self {
            Mode::Teleoperated => Control::TELEOP,
            Mode::Autonomous => Control::AUTO,
            Mode::Test => Control::TEST,
        }
    }
}

