//! PUBREC

use std::io::Read;

use crate::control::variable_header::PacketIdentifier;
use crate::control::{ControlType, FixedHeader, PacketType};
use crate::packet::{DecodablePacket, PacketError};
use crate::Decodable;

/// `PUBREC` packet
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PubrecPacket {
    fixed_header: FixedHeader,
    packet_identifier: PacketIdentifier,
}

encodable_packet!(PubrecPacket(packet_identifier));

impl PubrecPacket {
    pub fn new(pkid: u16) -> PubrecPacket {
        PubrecPacket {
            fixed_header: FixedHeader::new(PacketType::with_default(ControlType::PublishReceived), 2),
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

impl DecodablePacket for PubrecPacket {
    type Payload = ();

    fn decode_packet<R: Read>(reader: &mut R, fixed_header: FixedHeader) -> Result<Self, PacketError<Self>> {
        let packet_identifier: PacketIdentifier = PacketIdentifier::decode(reader)?;
        Ok(PubrecPacket {
            fixed_header,
            packet_identifier,
        })
    }
}
