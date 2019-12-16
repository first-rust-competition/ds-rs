use crate::proto::udp::inbound::UdpResponsePacket;
use crate::proto::udp::outbound::UdpControlPacket;
use bytes::BytesMut;
use failure::Fail;
use std::io;
use std::io::Error;
use tokio_util::codec::{Decoder, Encoder};

pub mod inbound;
pub mod outbound;

pub struct DsUdpCodec;

impl Decoder for DsUdpCodec {
    type Item = UdpResponsePacket;
    type Error = failure::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut buf = src.clone().freeze();

        use failure::bail;
        match UdpResponsePacket::decode(&mut buf) {
            Ok(packet) => Ok(Some(packet)),
            Err(e) => match e.downcast::<io::Error>() {
                Ok(e) => {
                    if e.kind() == io::ErrorKind::UnexpectedEof {
                        Ok(None)
                    } else {
                        Err(e.into())
                    }
                }
                _ => bail!("Failed to decode UDP packet."),
            },
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
