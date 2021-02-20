//! DISCONNECT

use std::io::Read;

use crate::control::{ControlType, FixedHeader, PacketType};
use crate::packet::{DecodablePacket, PacketError};

/// `DISCONNECT` packet
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DisconnectPacket {
    fixed_header: FixedHeader,
}

encodable_packet!(DisconnectPacket());

impl DisconnectPacket {
    pub fn new() -> DisconnectPacket {
        DisconnectPacket {
            fixed_header: FixedHeader::new(PacketType::with_default(ControlType::Disconnect), 0),
        }
    }
}

impl Default for DisconnectPacket {
    fn default() -> DisconnectPacket {
        DisconnectPacket::new()
    }
}

impl DecodablePacket for DisconnectPacket {
    type Payload = ();

    fn decode_packet<R: Read>(_reader: &mut R, fixed_header: FixedHeader) -> Result<Self, PacketError<Self>> {
        Ok(DisconnectPacket { fixed_header })
    }
}
