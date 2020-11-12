//! PINGRESP

use std::io::{Read, Write};

use crate::control::{ControlType, FixedHeader, PacketType};
use crate::packet::{Packet, PacketError};

/// `PINGRESP` packet
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PingrespPacket {
    fixed_header: FixedHeader,
    payload: (),
}

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

    fn fixed_header(&self) -> &FixedHeader {
        &self.fixed_header
    }

    fn payload(self) -> Self::Payload {
        self.payload
    }

    fn payload_ref(&self) -> &Self::Payload {
        &self.payload
    }

    fn encode_variable_headers<W: Write>(&self, _writer: &mut W) -> Result<(), PacketError<Self>> {
        Ok(())
    }

    fn encoded_variable_headers_length(&self) -> u32 {
        0
    }

    fn decode_packet<R: Read>(_reader: &mut R, fixed_header: FixedHeader) -> Result<Self, PacketError<Self>> {
        Ok(PingrespPacket {
            fixed_header,
            payload: (),
        })
    }
}
