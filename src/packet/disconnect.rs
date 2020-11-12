//! DISCONNECT

use std::io::{Read, Write};

use crate::control::{ControlType, FixedHeader, PacketType};
use crate::packet::{Packet, PacketError};

/// `DISCONNECT` packet
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DisconnectPacket {
    fixed_header: FixedHeader,
    payload: (),
}

impl DisconnectPacket {
    pub fn new() -> DisconnectPacket {
        DisconnectPacket {
            fixed_header: FixedHeader::new(PacketType::with_default(ControlType::Disconnect), 0),
            payload: (),
        }
    }
}

impl Default for DisconnectPacket {
    fn default() -> DisconnectPacket {
        DisconnectPacket::new()
    }
}

impl Packet for DisconnectPacket {
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
        Ok(DisconnectPacket {
            fixed_header,
            payload: (),
        })
    }
}
