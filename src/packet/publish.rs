//! PUBLISH

use std::io::{self, Read, Write};

use crate::control::{FixedHeader, PacketType};
use crate::packet::{DecodablePacket, PacketError};
use crate::qos::QualityOfService;
use crate::topic_name::TopicName;
use crate::{control::variable_header::PacketIdentifier, TopicNameRef};
use crate::{Decodable, Encodable};

use super::EncodablePacket;

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

    pub fn split(self) -> (QualityOfService, Option<u16>) {
        match self {
            QoSWithPacketIdentifier::Level0 => (QualityOfService::Level0, None),
            QoSWithPacketIdentifier::Level1(pkid) => (QualityOfService::Level1, Some(pkid)),
            QoSWithPacketIdentifier::Level2(pkid) => (QualityOfService::Level2, Some(pkid)),
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

encodable_packet!(PublishPacket(topic_name, packet_identifier, payload));

impl PublishPacket {
    pub fn new<P: Into<Vec<u8>>>(topic_name: TopicName, qos: QoSWithPacketIdentifier, payload: P) -> PublishPacket {
        let (qos, pkid) = qos.split();
        let mut pk = PublishPacket {
            fixed_header: FixedHeader::new(PacketType::publish(qos), 0),
            topic_name,
            packet_identifier: pkid.map(PacketIdentifier),
            payload: payload.into(),
        };
        pk.fix_header_remaining_len();
        pk
    }

    pub fn set_dup(&mut self, dup: bool) {
        self.fixed_header
            .packet_type
            .update_flags(|flags| (flags & !(1 << 3)) | (dup as u8) << 3)
    }

    pub fn dup(&self) -> bool {
        self.fixed_header.packet_type.flags() & 0x80 != 0
    }

    pub fn set_qos(&mut self, qos: QoSWithPacketIdentifier) {
        let (qos, pkid) = qos.split();
        self.fixed_header
            .packet_type
            .update_flags(|flags| (flags & !0b0110) | (qos as u8) << 1);
        self.packet_identifier = pkid.map(PacketIdentifier);
        self.fix_header_remaining_len();
    }

    pub fn qos(&self) -> QoSWithPacketIdentifier {
        match self.packet_identifier {
            None => QoSWithPacketIdentifier::Level0,
            Some(pkid) => {
                let qos_val = (self.fixed_header.packet_type.flags() & 0b0110) >> 1;
                match qos_val {
                    1 => QoSWithPacketIdentifier::Level1(pkid.0),
                    2 => QoSWithPacketIdentifier::Level2(pkid.0),
                    _ => unreachable!(),
                }
            }
        }
    }

    pub fn set_retain(&mut self, ret: bool) {
        self.fixed_header
            .packet_type
            .update_flags(|flags| (flags & !0b0001) | (ret as u8))
    }

    pub fn retain(&self) -> bool {
        self.fixed_header.packet_type.flags() & 0b0001 != 0
    }

    pub fn set_topic_name(&mut self, topic_name: TopicName) {
        self.topic_name = topic_name;
        self.fix_header_remaining_len();
    }

    pub fn topic_name(&self) -> &str {
        &self.topic_name[..]
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn set_payload<P: Into<Vec<u8>>>(&mut self, payload: P) {
        self.payload = payload.into();
        self.fix_header_remaining_len();
    }
}

impl DecodablePacket for PublishPacket {
    type DecodePacketError = std::convert::Infallible;

    fn decode_packet<R: Read>(reader: &mut R, fixed_header: FixedHeader) -> Result<Self, PacketError<Self>> {
        let topic_name = TopicName::decode(reader)?;

        let qos = (fixed_header.packet_type.flags() & 0b0110) >> 1;
        let packet_identifier = if qos > 0 {
            Some(PacketIdentifier::decode(reader)?)
        } else {
            None
        };

        let vhead_len =
            topic_name.encoded_length() + packet_identifier.as_ref().map(|x| x.encoded_length()).unwrap_or(0);
        let payload_len = fixed_header.remaining_length - vhead_len;

        let payload = Vec::<u8>::decode_with(reader, Some(payload_len))?;

        Ok(PublishPacket {
            fixed_header,
            topic_name,
            packet_identifier,
            payload,
        })
    }
}

/// `PUBLISH` packet by reference, for encoding only
pub struct PublishPacketRef<'a> {
    fixed_header: FixedHeader,
    topic_name: &'a TopicNameRef,
    packet_identifier: Option<PacketIdentifier>,
    payload: &'a [u8],
}

impl<'a> PublishPacketRef<'a> {
    pub fn new(topic_name: &'a TopicNameRef, qos: QoSWithPacketIdentifier, payload: &'a [u8]) -> PublishPacketRef<'a> {
        let (qos, pkid) = qos.split();

        let mut pk = PublishPacketRef {
            fixed_header: FixedHeader::new(PacketType::publish(qos), 0),
            topic_name,
            packet_identifier: pkid.map(PacketIdentifier),
            payload,
        };
        pk.fix_header_remaining_len();
        pk
    }

    fn fix_header_remaining_len(&mut self) {
        self.fixed_header.remaining_length =
            self.topic_name.encoded_length() + self.packet_identifier.encoded_length() + self.payload.encoded_length();
    }
}

impl EncodablePacket for PublishPacketRef<'_> {
    fn fixed_header(&self) -> &FixedHeader {
        &self.fixed_header
    }

    fn encode_packet<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.topic_name.encode(writer)?;
        self.packet_identifier.encode(writer)?;
        self.payload.encode(writer)
    }

    fn encoded_packet_length(&self) -> u32 {
        self.topic_name.encoded_length() + self.packet_identifier.encoded_length() + self.payload.encoded_length()
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

    #[test]
    fn issue56() {
        let mut packet = PublishPacket::new(
            TopicName::new("topic").unwrap(),
            QoSWithPacketIdentifier::Level0,
            Vec::new(),
        );
        assert_eq!(packet.fixed_header().remaining_length, 7);

        packet.set_qos(QoSWithPacketIdentifier::Level1(1));
        assert_eq!(packet.fixed_header().remaining_length, 9);
    }
}
