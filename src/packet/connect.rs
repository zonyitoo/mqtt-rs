//! CONNECT

use std::io::{Read, Write};

use control::variable_header::protocol_level::SPEC_3_1_1;
use control::variable_header::{ConnectFlags, KeepAlive, ProtocolLevel, ProtocolName};
use control::{ControlType, FixedHeader, PacketType};
use encodable::VarBytes;
use packet::{Packet, PacketError};
use topic_name::TopicName;
use {Decodable, Encodable};

/// `CONNECT` packet
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ConnectPacket {
    fixed_header: FixedHeader,
    protocol_name: ProtocolName,

    protocol_level: ProtocolLevel,
    flags: ConnectFlags,
    keep_alive: KeepAlive,

    payload: ConnectPacketPayload,
}

impl ConnectPacket {
    pub fn new<P, C>(protoname: P, client_identifier: C) -> ConnectPacket
    where
        P: Into<String>,
        C: Into<String>,
    {
        ConnectPacket::with_level(protoname, client_identifier, SPEC_3_1_1)
    }

    pub fn with_level<P, C>(protoname: P, client_identifier: C, level: u8) -> ConnectPacket
    where
        P: Into<String>,
        C: Into<String>,
    {
        let mut pk = ConnectPacket {
            fixed_header: FixedHeader::new(PacketType::with_default(ControlType::Connect), 0),
            protocol_name: ProtocolName(protoname.into()),
            protocol_level: ProtocolLevel(level),
            flags: ConnectFlags::empty(),
            keep_alive: KeepAlive(0),
            payload: ConnectPacketPayload::new(client_identifier.into()),
        };

        pk.fixed_header.remaining_length = pk.calculate_remaining_length();

        pk
    }

    #[inline]
    fn calculate_remaining_length(&self) -> u32 {
        self.encoded_variable_headers_length() + self.payload_ref().encoded_length()
    }

    pub fn set_keep_alive(&mut self, keep_alive: u16) {
        self.keep_alive = KeepAlive(keep_alive);
    }

    pub fn set_user_name(&mut self, name: Option<String>) {
        self.flags.user_name = name.is_some();
        self.payload.user_name = name;
        self.fixed_header.remaining_length = self.calculate_remaining_length();
    }

    pub fn set_will(&mut self, topic_message: Option<(TopicName, Vec<u8>)>) {
        self.flags.will_flag = topic_message.is_some();

        match topic_message {
            Some((topic, msg)) => {
                self.payload.will_topic = Some(topic);
                self.payload.will_message = Some(VarBytes(msg));
            }
            None => {
                self.payload.will_topic = None;
                self.payload.will_message = None;
            }
        }

        self.fixed_header.remaining_length = self.calculate_remaining_length();
    }

    pub fn set_password(&mut self, password: Option<String>) {
        self.flags.password = password.is_some();
        self.payload.password = password;
        self.fixed_header.remaining_length = self.calculate_remaining_length();
    }

    pub fn set_client_identifier<I: Into<String>>(&mut self, id: I) {
        self.payload.client_identifier = id.into();
        self.fixed_header.remaining_length = self.calculate_remaining_length();
    }

    pub fn set_will_retain(&mut self, will_retain: bool) {
        self.flags.will_retain = will_retain;
    }

    pub fn set_will_qos(&mut self, will_qos: u8) {
        assert!(will_qos <= 2);
        self.flags.will_qos = will_qos;
    }

    pub fn set_clean_session(&mut self, clean_session: bool) {
        self.flags.clean_session = clean_session;
    }

    pub fn user_name(&self) -> Option<&str> {
        self.payload.user_name.as_ref().map(|x| &x[..])
    }

    pub fn password(&self) -> Option<&str> {
        self.payload.password.as_ref().map(|x| &x[..])
    }

    pub fn will(&self) -> Option<(&str, &Vec<u8>)> {
        self.payload
            .will_topic
            .as_ref()
            .map(|x| &x[..])
            .and_then(|topic| self.payload.will_message.as_ref().map(|msg| (topic, &msg.0)))
    }

    pub fn will_retain(&self) -> bool {
        self.flags.will_retain
    }

    pub fn will_qos(&self) -> u8 {
        self.flags.will_qos
    }

    pub fn client_identifier(&self) -> &str {
        &self.payload.client_identifier[..]
    }

    pub fn clean_session(&self) -> bool {
        self.flags.clean_session
    }
}

impl Packet for ConnectPacket {
    type Payload = ConnectPacketPayload;

    fn fixed_header(&self) -> &FixedHeader {
        &self.fixed_header
    }

    fn payload(self) -> ConnectPacketPayload {
        self.payload
    }

    fn payload_ref(&self) -> &ConnectPacketPayload {
        &self.payload
    }

    fn encode_variable_headers<W: Write>(&self, writer: &mut W) -> Result<(), PacketError> {
        self.protocol_name.encode(writer)?;
        self.protocol_level.encode(writer)?;
        self.flags.encode(writer)?;
        self.keep_alive.encode(writer)?;

        Ok(())
    }

    fn encoded_variable_headers_length(&self) -> u32 {
        self.protocol_name.encoded_length()
            + self.protocol_level.encoded_length()
            + self.flags.encoded_length()
            + self.keep_alive.encoded_length()
    }

    fn decode_packet<R: Read>(reader: &mut R, fixed_header: FixedHeader) -> Result<Self, PacketError> {
        let protoname: ProtocolName = Decodable::decode(reader)?;
        let protocol_level: ProtocolLevel = Decodable::decode(reader)?;
        let flags: ConnectFlags = Decodable::decode(reader)?;
        let keep_alive: KeepAlive = Decodable::decode(reader)?;
        let payload: ConnectPacketPayload = Decodable::decode_with(reader, Some(flags))?;

        Ok(ConnectPacket {
            fixed_header: fixed_header,
            protocol_name: protoname,
            protocol_level: protocol_level,
            flags: flags,
            keep_alive: keep_alive,
            payload: payload,
        })
    }
}

/// Payloads for connect packet
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ConnectPacketPayload {
    client_identifier: String,
    will_topic: Option<TopicName>,
    will_message: Option<VarBytes>,
    user_name: Option<String>,
    password: Option<String>,
}

impl ConnectPacketPayload {
    pub fn new(client_identifier: String) -> ConnectPacketPayload {
        ConnectPacketPayload {
            client_identifier: client_identifier,
            will_topic: None,
            will_message: None,
            user_name: None,
            password: None,
        }
    }
}

impl Encodable for ConnectPacketPayload {
    type Err = PacketError;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), PacketError> {
        self.client_identifier.encode(writer)?;

        if let Some(ref will_topic) = self.will_topic {
            will_topic.encode(writer)?;
        }

        if let Some(ref will_message) = self.will_message {
            will_message.encode(writer)?;
        }

        if let Some(ref user_name) = self.user_name {
            user_name.encode(writer)?;
        }

        if let Some(ref password) = self.password {
            password.encode(writer)?;
        }

        Ok(())
    }

    fn encoded_length(&self) -> u32 {
        self.client_identifier.encoded_length()
            + self.will_topic.as_ref().map(|t| t.encoded_length()).unwrap_or(0)
            + self.will_message.as_ref().map(|t| t.encoded_length()).unwrap_or(0)
            + self.user_name.as_ref().map(|t| t.encoded_length()).unwrap_or(0)
            + self.password.as_ref().map(|t| t.encoded_length()).unwrap_or(0)
    }
}

impl Decodable for ConnectPacketPayload {
    type Err = PacketError;
    type Cond = ConnectFlags;

    fn decode_with<R: Read>(reader: &mut R, rest: Option<ConnectFlags>) -> Result<ConnectPacketPayload, PacketError> {
        let mut need_will_topic = false;
        let mut need_will_message = false;
        let mut need_user_name = false;
        let mut need_password = false;

        if let Some(r) = rest {
            need_will_topic = r.will_flag;
            need_will_message = r.will_flag;
            need_user_name = r.user_name;
            need_password = r.password;
        }

        let ident: String = Decodable::decode(reader)?;
        let topic = if need_will_topic {
            Some(Decodable::decode(reader)?)
        } else {
            None
        };
        let msg = if need_will_message {
            Some(Decodable::decode(reader)?)
        } else {
            None
        };
        let uname = if need_user_name {
            Some(Decodable::decode(reader)?)
        } else {
            None
        };
        let pwd = if need_password {
            Some(Decodable::decode(reader)?)
        } else {
            None
        };

        Ok(ConnectPacketPayload {
            client_identifier: ident,
            will_topic: topic,
            will_message: msg,
            user_name: uname,
            password: pwd,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::io::Cursor;

    use {Decodable, Encodable};

    #[test]
    fn test_connect_packet_encode_basic() {
        let packet = ConnectPacket::new("MQTT".to_owned(), "12345".to_owned());
        let expected = b"\x10\x11\x00\x04MQTT\x04\x00\x00\x00\x00\x0512345";

        let mut buf = Vec::new();
        packet.encode(&mut buf).unwrap();

        assert_eq!(&expected[..], &buf[..]);
    }

    #[test]
    fn test_connect_packet_decode_basic() {
        let encoded_data = b"\x10\x11\x00\x04MQTT\x04\x00\x00\x00\x00\x0512345";

        let mut buf = Cursor::new(&encoded_data[..]);
        let packet = ConnectPacket::decode(&mut buf).unwrap();

        let expected = ConnectPacket::new("MQTT".to_owned(), "12345".to_owned());
        assert_eq!(expected, packet);
    }

    #[test]
    fn test_connect_packet_user_name() {
        let mut packet = ConnectPacket::new("MQTT".to_owned(), "12345".to_owned());
        packet.set_user_name(Some("mqtt_player".to_owned()));

        let mut buf = Vec::new();
        packet.encode(&mut buf).unwrap();

        let mut decode_buf = Cursor::new(buf);
        let decoded_packet = ConnectPacket::decode(&mut decode_buf).unwrap();

        assert_eq!(packet, decoded_packet);
    }
}
