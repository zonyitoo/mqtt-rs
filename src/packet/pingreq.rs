//! PINGREQ

use std::io::Read;

use crate::control::{ControlType, FixedHeader, PacketType};
use crate::packet::{DecodablePacket, PacketError};

/// `PINGREQ` packet
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PingreqPacket {
    fixed_header: FixedHeader,
}

encodable_packet!(PingreqPacket());

impl PingreqPacket {
    pub fn new() -> PingreqPacket {
        PingreqPacket {
            fixed_header: FixedHeader::new(PacketType::with_default(ControlType::PingRequest), 0),
        }
    }
}

impl Default for PingreqPacket {
    fn default() -> PingreqPacket {
        PingreqPacket::new()
    }
}

impl DecodablePacket for PingreqPacket {
    type Payload = ();

    fn decode_packet<R: Read>(_reader: &mut R, fixed_header: FixedHeader) -> Result<Self, PacketError<Self>> {
        Ok(PingreqPacket { fixed_header })
    }
}
