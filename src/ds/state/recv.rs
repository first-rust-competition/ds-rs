use crate::ds::state::TcpConsumer;
use crate::TcpPacket;
use crate::proto::udp::inbound::types::*;

pub struct RecvState {
    battery_voltage: f32,
    trace: Trace,
}

pub struct TcpState {
    pub tcp_consumer: Option<Box<TcpConsumer>>
}

impl TcpState {
    pub fn new() -> TcpState {
        TcpState {
            tcp_consumer: None
        }
    }

    pub fn set_tcp_consumer(&mut self, consumer: impl FnMut(TcpPacket) + Send + Sync + 'static) {
        self.tcp_consumer = Some(Box::new(consumer));
    }
}

impl RecvState {
    pub fn new() -> RecvState {
        RecvState {
            battery_voltage: 0f32,
            trace: Trace::empty()
        }
    }

    pub fn battery_voltage(&self) -> f32 {
        self.battery_voltage
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
}