use std::io::{Read, Write};


use control::{FixedHeader, PacketType, ControlType};
use control::variable_header::{TopicName, PacketIdentifier};
use packet::{Packet, PacketError};
use {Encodable, Decodable};

#[derive(Debug, Eq, PartialEq)]
pub struct PublishPacket {
    fixed_header: FixedHeader,
    topic_name: TopicName,
    packet_identifier: PacketIdentifier,
    payload: Vec<u8>,
}

impl PublishPacket {
    pub fn new(topic_name: String, pkid: u16, payload: Vec<u8>) -> PublishPacket {
        let mut pk = PublishPacket {
            fixed_header: FixedHeader::new(PacketType::with_default(ControlType::Publish), 0),
            topic_name: TopicName(topic_name),
            packet_identifier: PacketIdentifier(pkid),
            payload: payload,
        };
        pk.fixed_header.remaining_length = pk.calculate_remaining_length();
        pk
    }

    #[inline]
    fn calculate_remaining_length(&self) -> u32 {
        self.encoded_variable_headers_length() +
            self.payload().encoded_length()
    }

    pub fn set_dup(&mut self, dup: bool) {
        self.fixed_header.packet_type.flags |= (dup as u8) << 3;
    }

    pub fn dup(&self) -> bool {
        self.fixed_header.packet_type.flags & 0x80 != 0
    }

    pub fn set_qos(&mut self, qos: u8) {
        assert!(qos <= 2);
        self.fixed_header.packet_type.flags |= qos << 1;
    }

    pub fn qos(&self) -> u8 {
        (self.fixed_header.packet_type.flags & 0x06) >> 1
    }

    pub fn set_retain(&mut self, ret: bool) {
        self.fixed_header.packet_type.flags |= ret as u8;
    }

    pub fn retain(&self) -> bool {
        self.fixed_header.packet_type.flags & 0x01 != 0
    }

    pub fn set_topic_name(&mut self, topic_name: String) {
        self.topic_name.0 = topic_name;
        self.fixed_header.remaining_length = self.calculate_remaining_length();
    }

    pub fn topic_name(&self) -> &str {
        &self.topic_name.0[..]
    }

    pub fn packet_identifier(&self) -> u16 {
        self.packet_identifier.0
    }

    pub fn set_packet_identifier(&mut self, pkid: u16) {
        self.packet_identifier.0 = pkid;
    }
}

impl<'a> Packet<'a> for PublishPacket {
    type Payload = Vec<u8>;

    fn fixed_header(&self) -> &FixedHeader {
        &self.fixed_header
    }

    fn payload(&self) -> &Self::Payload {
        &self.payload
    }

    fn encode_variable_headers<W: Write>(&self, writer: &mut W) -> Result<(), PacketError<'a, Self>> {
        try!(self.topic_name.encode(writer));
        try!(self.packet_identifier.encode(writer));

        Ok(())
    }

    fn encoded_variable_headers_length(&self) -> u32 {
        self.topic_name.encoded_length()
            + self.packet_identifier.encoded_length()
    }

    fn decode_packet<R: Read>(reader: &mut R, fixed_header: FixedHeader) -> Result<Self, PacketError<'a, Self>> {
        let topic_name: TopicName = try!(TopicName::decode(reader));
        let packet_identifier: PacketIdentifier = try!(PacketIdentifier::decode(reader));

        let vhead_len = topic_name.encoded_length() + packet_identifier.encoded_length();
        let payload_len = fixed_header.remaining_length - vhead_len;

        let payload: Vec<u8> = try!(Decodable::decode_with(reader, Some(payload_len)));

        Ok(PublishPacket {
            fixed_header: fixed_header,
            topic_name: topic_name,
            packet_identifier: packet_identifier,
            payload: payload,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::io::Cursor;

    use {Encodable, Decodable};

    #[test]
    fn test_publish_packet_basic() {
        let packet = PublishPacket::new("a/b".to_owned(), 10, b"Hello world!".to_vec());

        let mut buf = Vec::new();
        packet.encode(&mut buf).unwrap();

        let mut decode_buf = Cursor::new(buf);
        let decoded = PublishPacket::decode(&mut decode_buf).unwrap();

        assert_eq!(packet, decoded);
    }
}
