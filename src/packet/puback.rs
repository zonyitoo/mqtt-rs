//! PUBACK

use std::io::Read;

use crate::control::variable_header::PacketIdentifier;
use crate::control::{ControlType, FixedHeader, PacketType};
use crate::packet::{DecodablePacket, PacketError};
use crate::Decodable;

/// `PUBACK` packet
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PubackPacket {
    fixed_header: FixedHeader,
    packet_identifier: PacketIdentifier,
}

encodable_packet!(PubackPacket(packet_identifier));

impl PubackPacket {
    pub fn new(pkid: u16) -> PubackPacket {
        PubackPacket {
            fixed_header: FixedHeader::new(PacketType::with_default(ControlType::PublishAcknowledgement), 2),
            packet_identifier: PacketIdentifier(pkid),
        }
    }

    pub fn packet_identifier(&self) -> u16 {
        self.packet_identifier.0
    }

    pub fn set_packet_identifier(&mut self, pkid: u16) {
        self.packet_identifier.0 = pkid;
    }
}

impl DecodablePacket for PubackPacket {
    type DecodePacketError = std::convert::Infallible;

    fn decode_packet<R: Read>(reader: &mut R, fixed_header: FixedHeader) -> Result<Self, PacketError<Self>> {
        let packet_identifier: PacketIdentifier = PacketIdentifier::decode(reader)?;
        Ok(PubackPacket {
            fixed_header,
            packet_identifier,
        })
    }
}
