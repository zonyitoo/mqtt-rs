//! PINGRESP

use std::io::Read;

use crate::control::{ControlType, FixedHeader, PacketType};
use crate::packet::{DecodablePacket, PacketError};

/// `PINGRESP` packet
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PingrespPacket {
    fixed_header: FixedHeader,
}

encodable_packet!(PingrespPacket());

impl PingrespPacket {
    pub fn new() -> PingrespPacket {
        PingrespPacket {
            fixed_header: FixedHeader::new(PacketType::with_default(ControlType::PingResponse), 0),
        }
    }
}

impl Default for PingrespPacket {
    fn default() -> PingrespPacket {
        PingrespPacket::new()
    }
}

impl DecodablePacket for PingrespPacket {
    type Payload = ();

    fn decode_packet<R: Read>(_reader: &mut R, fixed_header: FixedHeader) -> Result<Self, PacketError<Self>> {
        Ok(PingrespPacket { fixed_header })
    }
}
