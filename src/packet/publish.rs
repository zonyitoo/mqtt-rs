//! PUBLISH

use std::io::{Read, Write};

use crate::control::variable_header::PacketIdentifier;
use crate::control::{ControlType, FixedHeader, PacketType};
use crate::packet::{Packet, PacketError};
use crate::qos::QualityOfService;
use crate::topic_name::TopicName;
use crate::{Decodable, Encodable};

/// QoS with identifier pairs
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub enum QoSWithPacketIdentifier {
    Level0,
    Level1(u16),
    Level2(u16),
}

impl QoSWithPacketIdentifier {
    pub fn new(qos: QualityOfService, id: u16) -> QoSWithPacketIdentifier {
        match (qos, id) {
            (QualityOfService::Level0, _) => QoSWithPacketIdentifier::Level0,
            (QualityOfService::Level1, id) => QoSWithPacketIdentifier::Level1(id),
            (QualityOfService::Level2, id) => QoSWithPacketIdentifier::Level2(id),
        }
    }
}

/// `PUBLISH` packet
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PublishPacket {
    fixed_header: FixedHeader,
    topic_name: TopicName,
    packet_identifier: Option<PacketIdentifier>,
    payload: Vec<u8>,
}

impl PublishPacket {
    pub fn new<P: Into<Vec<u8>>>(topic_name: TopicName, qos: QoSWithPacketIdentifier, payload: P) -> PublishPacket {
        let (qos, pkid) = match qos {
            QoSWithPacketIdentifier::Level0 => (0, None),
            QoSWithPacketIdentifier::Level1(pkid) => (1, Some(PacketIdentifier(pkid))),
            QoSWithPacketIdentifier::Level2(pkid) => (2, Some(PacketIdentifier(pkid))),
        };

        let mut pk = PublishPacket {
            fixed_header: FixedHeader::new(PacketType::with_default(ControlType::Publish), 0),
            topic_name,
            packet_identifier: pkid,
            payload: payload.into(),
        };
        pk.fixed_header.packet_type.flags |= qos << 1;
        pk.fixed_header.remaining_length = pk.calculate_remaining_length();
        pk
    }

    #[inline]
    fn calculate_remaining_length(&self) -> u32 {
        self.encoded_variable_headers_length() + self.payload_ref().encoded_length()
    }

    pub fn set_dup(&mut self, dup: bool) {
        self.fixed_header.packet_type.flags |= (dup as u8) << 3;
    }

    pub fn dup(&self) -> bool {
        self.fixed_header.packet_type.flags & 0x80 != 0
    }

    pub fn set_qos(&mut self, qos: QoSWithPacketIdentifier) {
        let (qos, pkid) = match qos {
            QoSWithPacketIdentifier::Level0 => (0, None),
            QoSWithPacketIdentifier::Level1(pkid) => (1, Some(PacketIdentifier(pkid))),
            QoSWithPacketIdentifier::Level2(pkid) => (2, Some(PacketIdentifier(pkid))),
        };
        self.fixed_header.packet_type.flags |= qos << 1;
        self.packet_identifier = pkid;
    }

    pub fn qos(&self) -> QoSWithPacketIdentifier {
        match self.packet_identifier {
            None => QoSWithPacketIdentifier::Level0,
            Some(pkid) => {
                let qos_val = (self.fixed_header.packet_type.flags & 0x06) >> 1;
                match qos_val {
                    1 => QoSWithPacketIdentifier::Level1(pkid.0),
                    2 => QoSWithPacketIdentifier::Level2(pkid.0),
                    _ => unreachable!(),
                }
            }
        }
    }

    pub fn set_retain(&mut self, ret: bool) {
        self.fixed_header.packet_type.flags |= ret as u8;
    }

    pub fn retain(&self) -> bool {
        self.fixed_header.packet_type.flags & 0x01 != 0
    }

    pub fn set_topic_name(&mut self, topic_name: TopicName) {
        self.topic_name = topic_name;
        self.fixed_header.remaining_length = self.calculate_remaining_length();
    }

    pub fn topic_name(&self) -> &str {
        &self.topic_name[..]
    }
}

impl Packet for PublishPacket {
    type Payload = Vec<u8>;

    fn fixed_header(&self) -> &FixedHeader {
        &self.fixed_header
    }

    fn payload(self) -> Self::Payload {
        self.payload
    }

    fn payload_ref(&self) -> &Self::Payload {
        &self.payload
    }

    fn encode_variable_headers<W: Write>(&self, writer: &mut W) -> Result<(), PacketError<Self>> {
        self.topic_name.encode(writer)?;

        if let Some(pkid) = self.packet_identifier.as_ref() {
            pkid.encode(writer)?;
        }

        Ok(())
    }

    fn encoded_variable_headers_length(&self) -> u32 {
        self.topic_name.encoded_length() + self.packet_identifier.as_ref().map(|x| x.encoded_length()).unwrap_or(0)
    }

    fn decode_packet<R: Read>(reader: &mut R, fixed_header: FixedHeader) -> Result<Self, PacketError<Self>> {
        let topic_name: TopicName = TopicName::decode(reader)?;

        let packet_identifier = if fixed_header.packet_type.flags & 0x06 != 0 {
            Some(PacketIdentifier::decode(reader)?)
        } else {
            None
        };

        let vhead_len =
            topic_name.encoded_length() + packet_identifier.as_ref().map(|x| x.encoded_length()).unwrap_or(0);
        let payload_len = fixed_header.remaining_length - vhead_len;

        let payload: Vec<u8> = Decodable::decode_with(reader, Some(payload_len))?;

        Ok(PublishPacket {
            fixed_header,
            topic_name,
            packet_identifier,
            payload,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::io::Cursor;

    use crate::topic_name::TopicName;
    use crate::{Decodable, Encodable};

    #[test]
    fn test_publish_packet_basic() {
        let packet = PublishPacket::new(
            TopicName::new("a/b".to_owned()).unwrap(),
            QoSWithPacketIdentifier::Level2(10),
            b"Hello world!".to_vec(),
        );

        let mut buf = Vec::new();
        packet.encode(&mut buf).unwrap();

        let mut decode_buf = Cursor::new(buf);
        let decoded = PublishPacket::decode(&mut decode_buf).unwrap();

        assert_eq!(packet, decoded);
    }
}
