//! CONNECT

use std::io::{self, Read, Write};

use crate::control::variable_header::protocol_level::SPEC_3_1_1;
use crate::control::variable_header::{ConnectFlags, KeepAlive, ProtocolLevel, ProtocolName, VariableHeaderError};
use crate::control::{ControlType, FixedHeader, PacketType};
use crate::encodable::VarBytes;
use crate::packet::{DecodablePacket, PacketError};
use crate::topic_name::{TopicName, TopicNameDecodeError, TopicNameError};
use crate::{Decodable, Encodable};

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

encodable_packet!(ConnectPacket(protocol_name, protocol_level, flags, keep_alive, payload));

impl ConnectPacket {
    pub fn new<C>(client_identifier: C) -> ConnectPacket
    where
        C: Into<String>,
    {
        ConnectPacket::with_level("MQTT", client_identifier, SPEC_3_1_1).expect("SPEC_3_1_1 should always be valid")
    }

    pub fn with_level<P, C>(protoname: P, client_identifier: C, level: u8) -> Result<ConnectPacket, VariableHeaderError>
    where
        P: Into<String>,
        C: Into<String>,
    {
        let protocol_level = ProtocolLevel::from_u8(level).ok_or(VariableHeaderError::InvalidProtocolVersion)?;
        let mut pk = ConnectPacket {
            fixed_header: FixedHeader::new(PacketType::with_default(ControlType::Connect), 0),
            protocol_name: ProtocolName(protoname.into()),
            protocol_level,
            flags: ConnectFlags::empty(),
            keep_alive: KeepAlive(0),
            payload: ConnectPacketPayload::new(client_identifier.into()),
        };

        pk.fix_header_remaining_len();

        Ok(pk)
    }

    pub fn set_keep_alive(&mut self, keep_alive: u16) {
        self.keep_alive = KeepAlive(keep_alive);
    }

    pub fn set_user_name(&mut self, name: Option<String>) {
        self.flags.user_name = name.is_some();
        self.payload.user_name = name;
        self.fix_header_remaining_len();
    }

    pub fn set_will(&mut self, topic_message: Option<(TopicName, Vec<u8>)>) {
        self.flags.will_flag = topic_message.is_some();

        self.payload.will = topic_message.map(|(t, m)| (t, VarBytes(m)));

        self.fix_header_remaining_len();
    }

    pub fn set_password(&mut self, password: Option<String>) {
        self.flags.password = password.is_some();
        self.payload.password = password;
        self.fix_header_remaining_len();
    }

    pub fn set_client_identifier<I: Into<String>>(&mut self, id: I) {
        self.payload.client_identifier = id.into();
        self.fix_header_remaining_len();
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

    pub fn will(&self) -> Option<(&str, &[u8])> {
        self.payload.will.as_ref().map(|(topic, msg)| (&topic[..], &*msg.0))
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

    pub fn protocol_name(&self) -> &str {
        &self.protocol_name.0
    }

    pub fn protocol_level(&self) -> ProtocolLevel {
        self.protocol_level
    }

    pub fn clean_session(&self) -> bool {
        self.flags.clean_session
    }

    /// Read back the "reserved" Connect flag bit 0. For compliant implementations this should
    /// always be false.
    pub fn reserved_flag(&self) -> bool {
        self.flags.reserved
    }
}

impl DecodablePacket for ConnectPacket {
    type DecodePacketError = ConnectPacketError;

    fn decode_packet<R: Read>(reader: &mut R, fixed_header: FixedHeader) -> Result<Self, PacketError<Self>> {
        let protoname: ProtocolName = Decodable::decode(reader)?;
        let protocol_level: ProtocolLevel = Decodable::decode(reader)?;
        let flags: ConnectFlags = Decodable::decode(reader)?;
        let keep_alive: KeepAlive = Decodable::decode(reader)?;
        let payload: ConnectPacketPayload =
            Decodable::decode_with(reader, Some(flags)).map_err(PacketError::PayloadError)?;

        Ok(ConnectPacket {
            fixed_header,
            protocol_name: protoname,
            protocol_level,
            flags,
            keep_alive,
            payload,
        })
    }
}

/// Payloads for connect packet
#[derive(Debug, Eq, PartialEq, Clone)]
struct ConnectPacketPayload {
    client_identifier: String,
    will: Option<(TopicName, VarBytes)>,
    user_name: Option<String>,
    password: Option<String>,
}

impl ConnectPacketPayload {
    pub fn new(client_identifier: String) -> ConnectPacketPayload {
        ConnectPacketPayload {
            client_identifier,
            will: None,
            user_name: None,
            password: None,
        }
    }
}

impl Encodable for ConnectPacketPayload {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        self.client_identifier.encode(writer)?;

        if let Some((will_topic, will_message)) = &self.will {
            will_topic.encode(writer)?;
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
            + self
                .will
                .as_ref()
                .map(|(a, b)| a.encoded_length() + b.encoded_length())
                .unwrap_or(0)
            + self.user_name.as_ref().map(|t| t.encoded_length()).unwrap_or(0)
            + self.password.as_ref().map(|t| t.encoded_length()).unwrap_or(0)
    }
}

impl Decodable for ConnectPacketPayload {
    type Error = ConnectPacketError;
    type Cond = Option<ConnectFlags>;

    fn decode_with<R: Read>(
        reader: &mut R,
        rest: Option<ConnectFlags>,
    ) -> Result<ConnectPacketPayload, ConnectPacketError> {
        let mut need_will = false;
        let mut need_user_name = false;
        let mut need_password = false;

        if let Some(r) = rest {
            need_will = r.will_flag;
            need_user_name = r.user_name;
            need_password = r.password;
        }

        let ident = String::decode(reader)?;
        let will = if need_will {
            let topic = TopicName::decode(reader).map_err(|e| match e {
                TopicNameDecodeError::IoError(e) => ConnectPacketError::from(e),
                TopicNameDecodeError::InvalidTopicName(e) => e.into(),
            })?;
            let msg = VarBytes::decode(reader)?;
            Some((topic, msg))
        } else {
            None
        };
        let uname = if need_user_name {
            Some(String::decode(reader)?)
        } else {
            None
        };
        let pwd = if need_password {
            Some(String::decode(reader)?)
        } else {
            None
        };

        Ok(ConnectPacketPayload {
            client_identifier: ident,
            will,
            user_name: uname,
            password: pwd,
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum ConnectPacketError {
    IoError(#[from] io::Error),
    TopicNameError(#[from] TopicNameError),
}

#[cfg(test)]
mod test {
    use super::*;

    use std::io::Cursor;

    use crate::{Decodable, Encodable};

    #[test]
    fn test_connect_packet_encode_basic() {
        let packet = ConnectPacket::new("12345".to_owned());
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

        let expected = ConnectPacket::new("12345".to_owned());
        assert_eq!(expected, packet);
    }

    #[test]
    fn test_connect_packet_user_name() {
        let mut packet = ConnectPacket::new("12345".to_owned());
        packet.set_user_name(Some("mqtt_player".to_owned()));

        let mut buf = Vec::new();
        packet.encode(&mut buf).unwrap();

        let mut decode_buf = Cursor::new(buf);
        let decoded_packet = ConnectPacket::decode(&mut decode_buf).unwrap();

        assert_eq!(packet, decoded_packet);
    }
}
