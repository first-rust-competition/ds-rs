use crate::inbound::udp::types::Status;
use crate::outbound::udp::UdpControlPacket;
use crate::outbound::udp::types::*;
use crate::outbound::udp::types::tags::*;
use crate::outbound::tcp::tags::*;

use std::f32;

type JoystickSupplier = Fn() -> Vec<JoystickValue> + Send + Sync + 'static;

/// The inner state of the driver station
/// contains information about the current mode, enabled status, and pending items for the next iteration of packets
pub struct State {
    mode: Mode,
    udp_seqnum: u16,
    enabled: bool,
    estopped: bool,
    alliance: Alliance,
    queued_tags: Vec<TagType>,
    joystick_provider: Option<Box<JoystickSupplier>>,
    pending_tcp: Vec<TcpTag>,
    battery_voltage: f32,
    pending_request: Option<Request>,
}

pub enum JoystickValue {
    Axis {
        id: u8,
        value: f32,
    },
    Button {
        id: u8,
        pressed: bool,
    },
}

impl State {
    pub fn new(alliance: Alliance) -> State {
        State {
            mode: Mode::Teleoperated,
            udp_seqnum: 1,
            enabled: false,
            estopped: false,
            alliance,
            joystick_provider: None,
            queued_tags: Vec::new(),
            pending_tcp: Vec::new(),
            battery_voltage: 0f32,
            pending_request: None,
        }
    }

    pub fn request(&mut self, request: Request) {
        self.pending_request = Some(request);
    }

    pub fn queue(&mut self, tag: TagType) {
        self.queued_tags.push(tag);
    }

    pub fn queue_tcp(&mut self, tag: TcpTag) {
        self.pending_tcp.push(tag);
    }

    pub fn pending_tcp(&self) -> &Vec<TcpTag> {
        &self.pending_tcp
    }

    pub fn pending_tcp_mut(&mut self) -> &mut Vec<TcpTag> {
        &mut self.pending_tcp
    }

    pub fn set_joystick_supplier<F>(&mut self, supplier: F)
        where F: Fn() -> Vec<JoystickValue> + Send + Sync + 'static
    {
        self.joystick_provider = Some(Box::new(supplier))
    }

    pub fn control(&mut self) -> UdpControlPacket {
        let mut axes = vec![0; 6];
        let mut buttons = vec![false; 10];

        if let Some(ref supplier) = &self.joystick_provider {
            for value in supplier() {
                match value {
                    JoystickValue::Button { id, pressed } => buttons.insert(id as usize, pressed),
                    JoystickValue::Axis { id, value } => {
                        let value = if (value - 1.0).abs() < f32::EPSILON {
                            127i8
                        } else {
                            (value * 128f32) as i8
                        };

                        axes.insert(id as usize, value);
                    }
                }
            }
        }

        self.queue(TagType::Joysticks(Joysticks::new(axes, buttons, vec![-1i16])));

        let mut control = self.mode.to_control();

        if self.enabled {
            control |= Control::ENABLED;
        }

        if self.estopped {
            control |= Control::ESTOP
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
            request: self.pending_request.take(),
            alliance: self.alliance,
            tags,
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

    /// Instructs the RIO to estop, disabling all outputs and disallowing
    pub fn estop(&mut self) {
        self.disable();
        self.estopped = true;
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
        } else if status & Status::AUTO == Status::AUTO {
            Some(Mode::Autonomous)
        } else if status & Status::TEST == Status::TEST {
            Some(Mode::Test)
        } else {
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

