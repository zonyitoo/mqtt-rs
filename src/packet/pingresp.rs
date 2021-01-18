//! PINGRESP

use std::io::Read;

use crate::control::{ControlType, FixedHeader, PacketType};
use crate::packet::{Packet, PacketError};

/// `PINGRESP` packet
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PingrespPacket {
    fixed_header: FixedHeader,
    payload: (),
}

encodable_packet!(PingrespPacket());

impl PingrespPacket {
    pub fn new() -> PingrespPacket {
        PingrespPacket {
            fixed_header: FixedHeader::new(PacketType::with_default(ControlType::PingResponse), 0),
            payload: (),
        }
    }
}

impl Default for PingrespPacket {
    fn default() -> PingrespPacket {
        PingrespPacket::new()
    }
}

impl Packet for PingrespPacket {
    type Payload = ();

    fn payload(self) -> Self::Payload {
        self.payload
    }

    fn payload_ref(&self) -> &Self::Payload {
        &self.payload
    }

    fn decode_packet<R: Read>(_reader: &mut R, fixed_header: FixedHeader) -> Result<Self, PacketError<Self>> {
        Ok(PingrespPacket {
            fixed_header,
            payload: (),
        })
    }
}
