use crate::proto::udp::inbound::UdpResponsePacket;
use crate::proto::udp::outbound::UdpControlPacket;
use bytes::{Buf, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

pub mod inbound;
pub mod outbound;

pub struct DsUdpCodec;

impl Decoder for DsUdpCodec {
    type Item = UdpResponsePacket;
    type Error = failure::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut buf = src.clone().freeze();

        match UdpResponsePacket::decode(&mut buf) {
            Ok((packet, len)) => {
                src.advance(len);
                return Ok(Some(packet));
            }
            Err(e) => Err(e),
        }
    }
}

impl Encoder for DsUdpCodec {
    type Item = UdpControlPacket;
    type Error = failure::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.extend(item.encode().iter());

        Ok(())
    }
}
