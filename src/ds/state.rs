use crate::inbound::udp::types::*;
use crate::inbound::tcp::TcpPacket;
use crate::outbound::udp::UdpControlPacket;
use crate::outbound::udp::types::*;
use crate::outbound::udp::types::tags::*;
use crate::outbound::tcp::*;

use super::JoystickValue;

use std::f32;

type JoystickSupplier = Fn() -> Vec<Vec<JoystickValue>> + Send + Sync + 'static;
type TcpConsumer = Fn(TcpPacket) + Send + Sync + 'static;

/// The inner state of the driver station
/// contains information about the current mode, enabled status, and pending items for the next iteration of packets
pub struct State {
    mode: Mode,
    udp_seqnum: u16,
    enabled: bool,
    estopped: bool,
    pub alliance: Alliance,
    pending_udp: Vec<UdpTag>,
    joystick_provider: Option<Box<JoystickSupplier>>,
    pub tcp_consumer: Option<Box<TcpConsumer>>,
    pending_tcp: Vec<TcpTag>,
    battery_voltage: f32,
    pending_request: Option<Request>,
    trace: Trace,
}



impl State {
    pub fn new(alliance: Alliance) -> State {
        State {
            mode: Mode::Teleoperated,
            udp_seqnum: 1,
            enabled: false,
            estopped: false,
            alliance,
            trace: Trace::empty(),
            battery_voltage: 0.0,
            joystick_provider: None,
            tcp_consumer: None,
            pending_udp: Vec::new(),
            pending_tcp: Vec::new(),
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

    pub fn queue_tcp(&mut self, tag: TcpTag) {
        self.pending_tcp.push(tag);
    }

    pub fn pending_tcp(&self) -> &Vec<TcpTag> {
        &self.pending_tcp
    }

    pub fn pending_tcp_mut(&mut self) -> &mut Vec<TcpTag> {
        &mut self.pending_tcp
    }

    pub fn set_joystick_supplier(&mut self, supplier: impl Fn() -> Vec<Vec<JoystickValue>> + Send + Sync + 'static) {
        self.joystick_provider = Some(Box::new(supplier))
    }

    pub fn set_tcp_consumer(&mut self, consumer: impl Fn(TcpPacket) + Send + Sync + 'static) {
        self.tcp_consumer = Some(Box::new(consumer));
    }

    pub fn set_alliance(&mut self, alliance: Alliance) {
        self.alliance = alliance;
    }

    pub fn battery_voltage(&self) -> &f32 {
        &self.battery_voltage
    }

    pub fn set_battery_voltage(&mut self, voltage: f32) {
        self.battery_voltage = voltage;
    }

    pub fn trace(&self) -> &Trace {
        &self.trace
    }

    pub fn set_trace(&mut self, trace: Trace) {
        self.trace = trace;
    }

    pub fn control(&mut self) -> UdpControlPacket {
        if let Some(ref supplier) = &self.joystick_provider {
            let joysticks: Vec<Vec<JoystickValue>> = supplier();

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

        // Control bits not related to the mode
        if self.enabled {
            control |= Control::ENABLED;
        }

        if self.estopped {
            control |= Control::ESTOP
        }

        // Hack to turn the enums into trait objects
        let mut tags: Vec<Box<Tag>> = Vec::new();

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

    pub fn enabled(&self) -> &bool {
        &self.enabled
    }

    /// Instructs the RIO to estop, disabling all outputs and disallowing
    pub fn estop(&mut self) {
        self.disable();
        self.estopped = true;
    }

    pub fn set_estop(&mut self, estop: bool) {
        self.estopped = estop;
    }

    pub fn estopped(&self) -> &bool {
        &self.estopped
    }
}

/// Represents the current Mode that the robot is in. the `Mode` of the robot is considered separately from whether it is enabled or not
#[derive(Copy, Clone)]
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

