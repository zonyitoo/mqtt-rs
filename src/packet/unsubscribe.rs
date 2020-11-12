//! UNSUBSCRIBE

use std::convert::From;
use std::error::Error;
use std::fmt;
use std::io::{self, Read, Write};
use std::string::FromUtf8Error;

use crate::control::variable_header::PacketIdentifier;
use crate::control::{ControlType, FixedHeader, PacketType};
use crate::encodable::StringEncodeError;
use crate::packet::{Packet, PacketError};
use crate::topic_filter::{TopicFilter, TopicFilterError};
use crate::{Decodable, Encodable};

/// `UNSUBSCRIBE` packet
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct UnsubscribePacket {
    fixed_header: FixedHeader,
    packet_identifier: PacketIdentifier,
    payload: UnsubscribePacketPayload,
}

impl UnsubscribePacket {
    pub fn new(pkid: u16, subscribes: Vec<TopicFilter>) -> UnsubscribePacket {
        let mut pk = UnsubscribePacket {
            fixed_header: FixedHeader::new(PacketType::with_default(ControlType::Unsubscribe), 0),
            packet_identifier: PacketIdentifier(pkid),
            payload: UnsubscribePacketPayload::new(subscribes),
        };
        pk.fixed_header.remaining_length = pk.encoded_variable_headers_length() + pk.payload.encoded_length();
        pk
    }

    pub fn packet_identifier(&self) -> u16 {
        self.packet_identifier.0
    }

    pub fn set_packet_identifier(&mut self, pkid: u16) {
        self.packet_identifier.0 = pkid;
    }
}

impl Packet for UnsubscribePacket {
    type Payload = UnsubscribePacketPayload;

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
        self.packet_identifier.encode(writer)?;

        Ok(())
    }

    fn encoded_variable_headers_length(&self) -> u32 {
        self.packet_identifier.encoded_length()
    }

    fn decode_packet<R: Read>(reader: &mut R, fixed_header: FixedHeader) -> Result<Self, PacketError<Self>> {
        let packet_identifier: PacketIdentifier = PacketIdentifier::decode(reader)?;
        let payload: UnsubscribePacketPayload = UnsubscribePacketPayload::decode_with(
            reader,
            Some(fixed_header.remaining_length - packet_identifier.encoded_length()),
        )
        .map_err(PacketError::PayloadError)?;
        Ok(UnsubscribePacket {
            fixed_header,
            packet_identifier,
            payload,
        })
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct UnsubscribePacketPayload {
    subscribes: Vec<TopicFilter>,
}

impl UnsubscribePacketPayload {
    pub fn new(subs: Vec<TopicFilter>) -> UnsubscribePacketPayload {
        UnsubscribePacketPayload { subscribes: subs }
    }

    pub fn subscribes(&self) -> &[TopicFilter] {
        &self.subscribes[..]
    }
}

impl Encodable for UnsubscribePacketPayload {
    type Err = UnsubscribePacketPayloadError;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), Self::Err> {
        for filter in self.subscribes.iter() {
            filter.encode(writer)?;
        }

        Ok(())
    }

    fn encoded_length(&self) -> u32 {
        self.subscribes.iter().fold(0, |b, a| b + a.encoded_length())
    }
}

impl Decodable for UnsubscribePacketPayload {
    type Err = UnsubscribePacketPayloadError;
    type Cond = u32;

    fn decode_with<R: Read>(
        reader: &mut R,
        payload_len: Option<u32>,
    ) -> Result<UnsubscribePacketPayload, UnsubscribePacketPayloadError> {
        let mut payload_len = payload_len.expect("Must provide payload length");
        let mut subs = Vec::new();

        while payload_len > 0 {
            let filter = TopicFilter::decode(reader)?;
            payload_len -= filter.encoded_length();
            subs.push(filter);
        }

        Ok(UnsubscribePacketPayload::new(subs))
    }
}

#[derive(Debug)]
pub enum UnsubscribePacketPayloadError {
    IoError(io::Error),
    FromUtf8Error(FromUtf8Error),
    StringEncodeError(StringEncodeError),
    TopicFilterError(TopicFilterError),
}

impl fmt::Display for UnsubscribePacketPayloadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UnsubscribePacketPayloadError::IoError(ref err) => err.fmt(f),
            UnsubscribePacketPayloadError::FromUtf8Error(ref err) => err.fmt(f),
            UnsubscribePacketPayloadError::StringEncodeError(ref err) => err.fmt(f),
            UnsubscribePacketPayloadError::TopicFilterError(ref err) => err.fmt(f),
        }
    }
}

impl Error for UnsubscribePacketPayloadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            UnsubscribePacketPayloadError::IoError(ref err) => Some(err),
            UnsubscribePacketPayloadError::FromUtf8Error(ref err) => Some(err),
            UnsubscribePacketPayloadError::StringEncodeError(ref err) => Some(err),
            UnsubscribePacketPayloadError::TopicFilterError(ref err) => Some(err),
        }
    }
}

impl From<StringEncodeError> for UnsubscribePacketPayloadError {
    fn from(err: StringEncodeError) -> UnsubscribePacketPayloadError {
        UnsubscribePacketPayloadError::StringEncodeError(err)
    }
}

impl From<io::Error> for UnsubscribePacketPayloadError {
    fn from(err: io::Error) -> UnsubscribePacketPayloadError {
        UnsubscribePacketPayloadError::IoError(err)
    }
}

impl From<TopicFilterError> for UnsubscribePacketPayloadError {
    fn from(err: TopicFilterError) -> UnsubscribePacketPayloadError {
        UnsubscribePacketPayloadError::TopicFilterError(err)
    }
}
