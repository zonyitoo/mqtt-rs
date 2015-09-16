use std::io::{self, Read, Write};
use std::error::Error;
use std::fmt;


use control::{FixedHeader, VariableHeader, PacketType, ControlType};
use control::variable_header::{ProtocolName, ProtocolLevel, ConnectFlags, KeepAlive};
use control::variable_header::protocol_level::SPEC_3_1_1;
use packet::{Packet, PacketError};
use {Encodable, Decodable};
use encodable::StringEncodeError;

#[derive(Debug, Eq, PartialEq)]
pub struct ConnectPacket {
    fixed_header: FixedHeader,
    variable_headers: Vec<VariableHeader>,
    payload: ConnectPacketPayload,
}

impl ConnectPacket {
    pub fn new(client_identifier: String) -> ConnectPacket {
        ConnectPacket::with_level(client_identifier, SPEC_3_1_1)
    }

    pub fn with_level(client_identifier: String, level: u8) -> ConnectPacket {
        let mut pk = ConnectPacket {
            fixed_header: FixedHeader::new(PacketType::with_default(ControlType::Connect), 0),
            variable_headers: vec![
                VariableHeader::new(ProtocolName("MQTT".to_owned())),
                VariableHeader::new(ProtocolLevel(level)),
                VariableHeader::new(ConnectFlags::empty()),
                VariableHeader::new(KeepAlive(0)),
            ],
            payload: ConnectPacketPayload::new(client_identifier),
        };

        pk.fixed_header.remaining_length = pk.calculate_remaining_length();

        pk
    }

    #[inline]
    fn calculate_remaining_length(&self) -> u32 {
        self.variable_headers().iter().fold(0, |b, a| b + a.encoded_length()) +
            self.payload().encoded_length()
    }

    pub fn set_user_name(&mut self, name: Option<String>) {
        if let &mut VariableHeader::ConnectFlags(ref mut flags) = &mut self.variable_headers[2] {
            flags.user_name = name.is_some();
        } else {
            panic!("Could not find connect flags variable header");
        }
        self.payload.user_name = name;
        self.fixed_header.remaining_length = self.calculate_remaining_length();
    }

    pub fn set_will(&mut self, topic_message: Option<(String, String)>) {
        if let &mut VariableHeader::ConnectFlags(ref mut flags) = &mut self.variable_headers[2] {
            flags.will_flag = topic_message.is_some();
        } else {
            panic!("Could not find connect flags variable header");
        }

        match topic_message {
            Some((topic, msg)) => {
                self.payload.will_topic = Some(topic);
                self.payload.will_message = Some(msg);
            },
            None => {
                self.payload.will_topic = None;
                self.payload.will_message = None;
            }
        }

        self.fixed_header.remaining_length = self.calculate_remaining_length();
    }

    pub fn set_password(&mut self, password: Option<String>) {
        if let &mut VariableHeader::ConnectFlags(ref mut flags) = &mut self.variable_headers[2] {
            flags.password = password.is_some();
        } else {
            panic!("Could not find connect flags variable header");
        }
        self.payload.password = password;
        self.fixed_header.remaining_length = self.calculate_remaining_length();
    }

    pub fn set_client_identifier(&mut self, id: String) {
        self.payload.client_identifier = id;
        self.fixed_header.remaining_length = self.calculate_remaining_length();
    }

    pub fn set_will_retain(&mut self, will_retain: bool) {
        if let &mut VariableHeader::ConnectFlags(ref mut flags) = &mut self.variable_headers[2] {
            flags.will_retain = will_retain;
        } else {
            panic!("Could not find connect flags variable header");
        }
    }

    pub fn set_will_qos(&mut self, will_qos: u8) {
        assert!(will_qos <= 2);
        if let &mut VariableHeader::ConnectFlags(ref mut flags) = &mut self.variable_headers[2] {
            flags.will_qos = will_qos;
        } else {
            panic!("Could not find connect flags variable header");
        }
    }

    pub fn set_clean_session(&mut self, clean_session: bool) {
        if let &mut VariableHeader::ConnectFlags(ref mut flags) = &mut self.variable_headers[2] {
            flags.clean_session = clean_session;
        } else {
            panic!("Could not find connect flags variable header");
        }
    }

    pub fn user_name(&self) -> Option<&str> {
        self.payload.user_name.as_ref().map(|x| &x[..])
    }

    pub fn password(&self) -> Option<&str> {
        self.payload.password.as_ref().map(|x| &x[..])
    }

    pub fn will(&self) -> Option<(&str, &str)> {
        self.payload.will_topic.as_ref().map(|x| &x[..])
            .and_then(|topic| self.payload.will_message.as_ref().map(|x| &x[..])
                             .map(|msg| (topic, msg)))
    }

    pub fn will_retain(&self) -> bool {
        if let &VariableHeader::ConnectFlags(ref flags) = &self.variable_headers[2] {
            flags.will_retain
        } else {
            panic!("Could not find connect flags variable header");
        }
    }

    pub fn will_qos(&self) -> u8 {
        if let &VariableHeader::ConnectFlags(ref flags) = &self.variable_headers[2] {
            flags.will_qos
        } else {
            panic!("Could not find connect flags variable header");
        }
    }

    pub fn client_identifier(&self) -> &str {
        &self.payload.client_identifier[..]
    }

    pub fn clean_session(&self) -> bool {
        if let &VariableHeader::ConnectFlags(ref flags) = &self.variable_headers[2] {
            flags.clean_session
        } else {
            panic!("Could not find connect flags variable header");
        }
    }
}

impl<'a> Packet<'a> for ConnectPacket {
    type Payload = ConnectPacketPayload;

    fn fixed_header(&self) -> &FixedHeader {
        &self.fixed_header
    }

    fn variable_headers(&self) -> &[VariableHeader] {
        &self.variable_headers[..]
    }

    fn payload(&self) -> &ConnectPacketPayload {
        &self.payload
    }

    fn decode_packet<R: Read>(reader: &mut R, fixed_header: FixedHeader) -> Result<Self, PacketError<'a, Self>> {
        let vheaders: Vec<VariableHeader> = vec![
            VariableHeader::ProtocolName(try!(Decodable::decode(reader))),
            VariableHeader::ProtocolLevel(try!(Decodable::decode(reader))),
            VariableHeader::ConnectFlags(try!(Decodable::decode(reader))),
            VariableHeader::KeepAlive(try!(Decodable::decode(reader))),
        ];
        let payload: ConnectPacketPayload =
            try!(Decodable::decode_with(reader, Some(&vheaders[..]))
                    .map_err(PacketError::PayloadError));

        Ok(ConnectPacket {
            fixed_header: fixed_header,
            variable_headers: vheaders,
            payload: payload,
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ConnectPacketPayload {
    client_identifier: String,
    will_topic: Option<String>,
    will_message: Option<String>,
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

impl<'a> Encodable<'a> for ConnectPacketPayload {
    type Err = ConnectPacketPayloadError;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), ConnectPacketPayloadError> {
        try!(self.client_identifier.encode(writer));

        if let Some(ref will_topic) = self.will_topic {
            try!(will_topic.encode(writer));
        }

        if let Some(ref will_message) = self.will_message {
            try!(will_message.encode(writer));
        }

        if let Some(ref user_name) = self.user_name {
            try!(user_name.encode(writer));
        }

        if let Some(ref password) = self.password {
            try!(password.encode(writer));
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

impl<'a> Decodable<'a> for ConnectPacketPayload {
    type Err = ConnectPacketPayloadError;
    type Cond = &'a [VariableHeader];

    fn decode_with<R: Read>(reader: &mut R, rest: Option<&[VariableHeader]>)
            -> Result<ConnectPacketPayload, ConnectPacketPayloadError> {
        let mut need_will_topic = false;
        let mut need_will_message = false;
        let mut need_user_name = false;
        let mut need_password = false;

        if let Some(r) = rest {
            for re in r.iter() {
                match re {
                    &VariableHeader::ConnectFlags(flags) => {
                        need_will_topic = flags.will_flag;
                        need_will_message = flags.will_flag;
                        need_user_name = flags.user_name;
                        need_password = flags.password;
                    },
                    _ => {}
                }
            }
        }

        let ident: String = try!(Decodable::decode(reader));
        let topic = if need_will_topic {
            Some(try!(Decodable::decode(reader)))
        } else {
            None
        };
        let msg = if need_will_message {
            Some(try!(Decodable::decode(reader)))
        } else {
            None
        };
        let uname = if need_user_name {
            Some(try!(Decodable::decode(reader)))
        } else {
            None
        };
        let pwd = if need_password {
            Some(try!(Decodable::decode(reader)))
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

#[derive(Debug)]
pub enum ConnectPacketPayloadError {
    IoError(io::Error),
    StringEncodeError(StringEncodeError),
}

impl fmt::Display for ConnectPacketPayloadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ConnectPacketPayloadError::IoError(ref err) => err.fmt(f),
            &ConnectPacketPayloadError::StringEncodeError(ref err) => err.fmt(f),
        }
    }
}

impl Error for ConnectPacketPayloadError {
    fn description(&self) -> &str {
        match self {
            &ConnectPacketPayloadError::IoError(ref err) => err.description(),
            &ConnectPacketPayloadError::StringEncodeError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match self {
            &ConnectPacketPayloadError::IoError(ref err) => Some(err),
            &ConnectPacketPayloadError::StringEncodeError(ref err) => Some(err),
        }
    }
}

impl From<StringEncodeError> for ConnectPacketPayloadError {
    fn from(err: StringEncodeError) -> ConnectPacketPayloadError {
        ConnectPacketPayloadError::StringEncodeError(err)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::io::Cursor;

    use {Encodable, Decodable};

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
