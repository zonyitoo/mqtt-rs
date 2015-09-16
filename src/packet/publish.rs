use std::io::Read;


use control::{FixedHeader, VariableHeader, PacketType, ControlType};
use control::variable_header::{TopicName, PacketIdentifier};
use packet::{Packet, PacketError};
use {Encodable, Decodable};

#[derive(Debug, Eq, PartialEq)]
pub struct PublishPacket {
    fixed_header: FixedHeader,
    variable_headers: Vec<VariableHeader>,
    payload: Vec<u8>,
}

impl PublishPacket {
    pub fn new(topic_name: String, pkid: u16, payload: Vec<u8>) -> PublishPacket {
        let mut pk = PublishPacket {
            fixed_header: FixedHeader::new(PacketType::with_default(ControlType::Publish), 0),
            variable_headers: vec![
                VariableHeader::new(TopicName(topic_name)),
                VariableHeader::new(PacketIdentifier(pkid)),
            ],
            payload: payload,
        };
        pk.fixed_header.remaining_length = pk.calculate_remaining_length();
        pk
    }

    #[inline]
    fn calculate_remaining_length(&self) -> u32 {
        self.variable_headers().iter().fold(0, |b, a| b + a.encoded_length()) +
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
        if let &mut VariableHeader::TopicName(ref mut tp_name) = &mut self.variable_headers[0] {
            *tp_name = TopicName(topic_name);
        } else {
            panic!("Could not find topic name variable header");
        }
        self.fixed_header.remaining_length = self.calculate_remaining_length();
    }

    pub fn topic_name(&self) -> &str {
        if let &VariableHeader::TopicName(ref tp_name) = &self.variable_headers[0] {
            &tp_name.0[..]
        } else {
            panic!("Could not find topic name variable header");
        }
    }

    pub fn packet_identifier(&self) -> u16 {
        if let &VariableHeader::PacketIdentifier(ref id) = &self.variable_headers[1] {
            id.0
        } else {
            panic!("Could not find packet identifier variable header");
        }
    }

    pub fn set_packet_identifier(&mut self, pkid: u16) {
        if let &mut VariableHeader::PacketIdentifier(ref mut id) = &mut self.variable_headers[1] {
            *id = PacketIdentifier(pkid);
        } else {
            panic!("Could not find packet identifier variable header");
        }
    }
}

impl<'a> Packet<'a> for PublishPacket {
    type Payload = Vec<u8>;

    fn fixed_header(&self) -> &FixedHeader {
        &self.fixed_header
    }

    fn variable_headers(&self) -> &[VariableHeader] {
        &self.variable_headers[..]
    }

    fn payload(&self) -> &Self::Payload {
        &self.payload
    }

    fn decode_packet<R: Read>(reader: &mut R, fixed_header: FixedHeader) -> Result<Self, PacketError<'a, Self>> {
        let topic_name: TopicName = try!(TopicName::decode(reader));
        let packet_identifier: PacketIdentifier = try!(PacketIdentifier::decode(reader));

        let vhead_len = topic_name.encoded_length() + packet_identifier.encoded_length();
        let payload_len = fixed_header.remaining_length - vhead_len;

        let payload: Vec<u8> = try!(Decodable::decode_with(reader, Some(payload_len)));

        Ok(PublishPacket {
            fixed_header: fixed_header,
            variable_headers: vec![
                VariableHeader::new(topic_name),
                VariableHeader::new(packet_identifier),
            ],
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
