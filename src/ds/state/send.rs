use crate::ds::state::JoystickSupplier;
use crate::proto::udp::outbound::types::tags::*;
use crate::proto::udp::outbound::types::{Control, Request};
use crate::proto::udp::outbound::*;
use crate::{Alliance, JoystickValue, Mode};
use std::f32;

pub struct SendState {
    mode: Mode,
    udp_seqnum: u16,
    enabled: bool,
    estopped: bool,
    pub alliance: Alliance,
    pending_udp: Vec<UdpTag>,
    joystick_provider: Option<Box<JoystickSupplier>>,
    pending_request: Option<Request>,
}

impl SendState {
    pub fn new(alliance: Alliance) -> SendState {
        SendState {
            mode: Mode::Autonomous,
            udp_seqnum: 0,
            enabled: false,
            estopped: false,
            alliance,
            pending_udp: Vec::new(),
            joystick_provider: None,
            pending_request: None,
        }
    }

    pub fn request(&mut self, request: Request) {
        self.pending_request = Some(request);
    }

    pub fn queue_udp(&mut self, tag: UdpTag) {
        self.pending_udp.push(tag);
    }

    pub fn pending_udp(&self) -> &Vec<UdpTag> {
        &self.pending_udp
    }

    pub fn set_joystick_supplier(
        &mut self,
        supplier: impl Fn() -> Vec<Vec<JoystickValue>> + Send + Sync + 'static,
    ) {
        self.joystick_provider = Some(Box::new(supplier))
    }

    pub fn set_alliance(&mut self, alliance: Alliance) {
        self.alliance = alliance;
    }

    pub fn control(&mut self) -> UdpControlPacket {
        if let Some(ref supplier) = &self.joystick_provider {
            let joysticks = supplier();

            // Joystick tags come one after another, iterate over the outer Vec and queue with each loop
            for i in 0..joysticks.len() {
                let mut axes = vec![0; 6];
                let mut buttons = vec![false; 10];
                let mut povs = vec![-1i16];

                for value in &joysticks[i] {
                    // If statements bound check to stop it from crashing
                    match value {
                        JoystickValue::Button { id, pressed } => {
                            if *id >= 1 && *id <= 10 {
                                let id = id - 1;
                                buttons.remove(id as usize);
                                buttons.insert(id as usize, *pressed)
                            }
                        }
                        JoystickValue::Axis { id, value } => {
                            if *id <= 5 {
                                let value = if (*value - 1.0).abs() < f32::EPSILON {
                                    127i8
                                } else {
                                    (value * 128f32) as i8
                                };

                                axes.remove(*id as usize);
                                axes.insert(*id as usize, value);
                            }
                        }
                        JoystickValue::POV { id, angle } => {
                            if *id == 0 {
                                povs.remove(*id as usize);
                                povs.insert(*id as usize, *angle);
                            }
                        }
                    }
                }
                self.queue_udp(UdpTag::Joysticks(Joysticks::new(axes, buttons, povs)));
            }
        }

        let mut control = self.mode.to_control();

        if self.enabled {
            control |= Control::ENABLED;
        }

        if self.estopped {
            control |= Control::ESTOP
        }

        let mut tags: Vec<Box<dyn Tag>> = Vec::new();

        for tag in self.pending_udp.clone() {
            match tag {
                UdpTag::Timezone(tz) => tags.push(Box::new(tz)),
                UdpTag::DateTime(dt) => tags.push(Box::new(dt)),
                UdpTag::Joysticks(joy) => tags.push(Box::new(joy)),
                UdpTag::Countdown(cnt) => tags.push(Box::new(cnt)),
            }
        }

        self.pending_udp.clear();

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
        self.udp_seqnum = self.udp_seqnum.wrapping_add(1);
    }

    pub fn reset_seqnum(&mut self) {
        self.udp_seqnum = 0;
    }

    #[allow(unused)]
    pub fn seqnum(&self) -> u16 {
        self.udp_seqnum
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn estop(&mut self) {
        self.disable();
        self.estopped = true;
    }

    pub fn estopped(&self) -> bool {
        self.estopped
    }
}
