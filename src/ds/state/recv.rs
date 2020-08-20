use crate::ds::state::TcpConsumer;
use crate::proto::tcp::outbound::TcpTag;
use crate::proto::udp::inbound::types::*;
use crate::Result;
use crate::TcpPacket;
use failure::format_err;
use futures_channel::mpsc::UnboundedSender;

/// All the data received from roboRIO UDP status packets that isn't already encoded in the send state
pub struct RecvState {
    /// The current battery voltage
    battery_voltage: f32,
    /// A bitflags struct that can be used to query the state of various aspects of the RIO
    trace: Trace,
}

impl RecvState {
    pub fn reset(&mut self) {
        self.battery_voltage = 0f32;
        self.trace = Trace::empty();
    }
}

/// All the state associated with TCP communication with the RIO
pub struct TcpState {
    /// An optional callback that should be notified upon incoming packets being decoded
    pub tcp_consumer: Option<Box<TcpConsumer>>,
    /// A channel of packets that should be sent to the roboRIO
    pending_tcp: Option<UnboundedSender<TcpTag>>,
}

impl TcpState {
    pub fn new() -> TcpState {
        TcpState {
            tcp_consumer: None,
            pending_tcp: None,
        }
    }

    pub fn queue_tcp(&self, tag: TcpTag) -> Result<()> {
        // pending_tcp is set by the tcp_conn function when it connects.
        self.pending_tcp
            .clone()
            .ok_or_else(|| format_err!("TCP task not spawned."))
            .and_then(move |tx| tx.unbounded_send(tag).map_err(|e| e.into()))
            .map(|_| ())
    }

    pub fn set_tcp_tx(&mut self, tx: Option<UnboundedSender<TcpTag>>) {
        self.pending_tcp = tx;
    }

    pub fn set_tcp_consumer(&mut self, consumer: impl FnMut(TcpPacket) + Send + Sync + 'static) {
        self.tcp_consumer = Some(Box::new(consumer));
    }
}

impl RecvState {
    pub fn new() -> RecvState {
        RecvState {
            battery_voltage: 0f32,
            trace: Trace::empty(),
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
