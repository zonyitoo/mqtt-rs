//! PUBREL

use std::io::Read;

use crate::control::variable_header::PacketIdentifier;
use crate::control::{ControlType, FixedHeader, PacketType};
use crate::packet::{DecodablePacket, PacketError};
use crate::Decodable;

/// `PUBREL` packet
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PubrelPacket {
    fixed_header: FixedHeader,
    packet_identifier: PacketIdentifier,
}

encodable_packet!(PubrelPacket(packet_identifier));

impl PubrelPacket {
    pub fn new(pkid: u16) -> PubrelPacket {
        PubrelPacket {
            fixed_header: FixedHeader::new(PacketType::with_default(ControlType::PublishRelease), 2),
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

impl DecodablePacket for PubrelPacket {
    type Payload = ();

    fn decode_packet<R: Read>(reader: &mut R, fixed_header: FixedHeader) -> Result<Self, PacketError<Self>> {
        let packet_identifier: PacketIdentifier = PacketIdentifier::decode(reader)?;
        Ok(PubrelPacket {
            fixed_header,
            packet_identifier,
        })
    }
}
