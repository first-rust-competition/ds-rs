use crate::ext::BufExt;
use crate::{Stdout, TcpPacket};
use bytes::{Buf, BytesMut};
use std::io;
use tokio_util::codec::{Decoder, Encoder};

pub mod inbound;
pub mod outbound;

pub struct DsTcpCodec;

impl Encoder for DsTcpCodec {
    type Item = ();
    type Error = failure::Error;

    fn encode(&mut self, _item: Self::Item, _dst: &mut BytesMut) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl Decoder for DsTcpCodec {
    type Item = TcpPacket;
    type Error = failure::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut buf = src.clone().freeze();

        fn inner(buf: &mut impl Buf) -> crate::Result<(TcpPacket, usize)> {
            let len = buf.read_u16_be()?;

            let id = buf.read_u8()?;
            match id {
                0x0c => {
                    Ok((TcpPacket::Stdout(Stdout::decode(buf, len as usize - 1)?), len as usize + 2))
                },
                _ => {
                    for _ in 0..(len - 1) {
                        let _ = buf.read_u8()?;
                    }
                    Ok((TcpPacket::Dummy, len as usize + 2))
                }
            }
        }

        use failure::bail;
        match inner(&mut buf) {
            Ok((packet, n)) => {
                src.advance(n);
                Ok(Some(packet))
            }
            Err(e) => match e.downcast::<io::Error>() {
                Ok(e) => {
                    if e.kind() == io::ErrorKind::UnexpectedEof {
                        Ok(None)
                    } else {
                        Err(e.into())
                    }
                }
                _ => bail!("Failed to decode TCP packet"),
            },
        }
    }
}
