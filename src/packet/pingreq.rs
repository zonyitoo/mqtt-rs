//! PINGREQ

use std::io::Read;

use crate::control::{ControlType, FixedHeader, PacketType};
use crate::packet::{Packet, PacketError};

/// `PINGREQ` packet
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PingreqPacket {
    fixed_header: FixedHeader,
    payload: (),
}

encodable_packet!(PingreqPacket());

impl PingreqPacket {
    pub fn new() -> PingreqPacket {
        PingreqPacket {
            fixed_header: FixedHeader::new(PacketType::with_default(ControlType::PingRequest), 0),
            payload: (),
        }
    }
}

impl Default for PingreqPacket {
    fn default() -> PingreqPacket {
        PingreqPacket::new()
    }
}

impl Packet for PingreqPacket {
    type Payload = ();

    fn payload(self) -> Self::Payload {
        self.payload
    }

    fn payload_ref(&self) -> &Self::Payload {
        &self.payload
    }

    fn decode_packet<R: Read>(_reader: &mut R, fixed_header: FixedHeader) -> Result<Self, PacketError<Self>> {
        Ok(PingreqPacket {
            fixed_header,
            payload: (),
        })
    }
}
