//! PUBREC

use std::io::{Read, Write};

use control::variable_header::PacketIdentifier;
use control::{ControlType, FixedHeader, PacketType};
use packet::{Packet, PacketError};
use {Decodable, Encodable};

/// `PUBREC` packet
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PubrecPacket {
    fixed_header: FixedHeader,
    packet_identifier: PacketIdentifier,
    payload: (),
}

impl PubrecPacket {
    pub fn new(pkid: u16) -> PubrecPacket {
        PubrecPacket {
            fixed_header: FixedHeader::new(PacketType::with_default(ControlType::PublishReceived), 2),
            packet_identifier: PacketIdentifier(pkid),
            payload: (),
        }
    }

    pub fn packet_identifier(&self) -> u16 {
        self.packet_identifier.0
    }

    pub fn set_packet_identifier(&mut self, pkid: u16) {
        self.packet_identifier.0 = pkid;
    }
}

impl Packet for PubrecPacket {
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

    fn encode_variable_headers<W: Write>(&self, writer: &mut W) -> Result<(), PacketError> {
        self.packet_identifier.encode(writer)?;

        Ok(())
    }

    fn encoded_variable_headers_length(&self) -> u32 {
        self.packet_identifier.encoded_length()
    }

    fn decode_packet<R: Read>(reader: &mut R, fixed_header: FixedHeader) -> Result<Self, PacketError> {
        let packet_identifier: PacketIdentifier = PacketIdentifier::decode(reader)?;
        Ok(PubrecPacket {
            fixed_header: fixed_header,
            packet_identifier: packet_identifier,
            payload: (),
        })
    }
}
