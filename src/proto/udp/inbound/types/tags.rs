use crate::ext::BufExt;
use crate::util::InboundTag;
use crate::Result;
use bytes::Buf;

macro_rules! gen_stub_tags {
    ($($struct_name:ident : $num_bytes:expr),*) => {
        $(
        pub(crate) struct $struct_name {
            _data: [u8; $num_bytes]
        }

        impl InboundTag for $struct_name {
            fn chomp(buf: &mut impl Buf) -> Result<Self> {
                let mut _data = [0; $num_bytes];

                for i in 0..$num_bytes {
                    _data[i] = buf.read_u8()?;
                }

                Ok($struct_name { _data })
            }
        }
        )*
    }
}

gen_stub_tags!(PDPLog : 25, JoystickOutput : 8, DiskInfo : 4, CPUInfo : 20, RAMInfo : 8, Unknown : 9, CANMetrics : 14);
