//! PINGREQ

use std::io::{Read, Write};

use control::{ControlType, FixedHeader, PacketType};
use packet::{Packet, PacketError};

/// `PINGREQ` packet
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PingreqPacket {
    fixed_header: FixedHeader,
    payload: (),
}

impl PingreqPacket {
    pub fn new() -> PingreqPacket {
        PingreqPacket {
            fixed_header: FixedHeader::new(PacketType::with_default(ControlType::PingRequest), 0),
            payload: (),
        }
    }
}

impl Packet for PingreqPacket {
    type Payload = ();

    fn fixed_header(&self) -> &FixedHeader {
        &self.fixed_header
    }

    fn payload(self) -> Self::Payload {
        self.payload
    }

    fn payload_ref(&self) -> &Self::Payload {
        &self.payload
    }

    fn encode_variable_headers<W: Write>(&self, _writer: &mut W) -> Result<(), PacketError> {
        Ok(())
    }

    fn encoded_variable_headers_length(&self) -> u32 {
        0
    }

    fn decode_packet<R: Read>(_reader: &mut R, fixed_header: FixedHeader) -> Result<Self, PacketError> {
        Ok(PingreqPacket {
            fixed_header: fixed_header,
            payload: (),
        })
    }
}
