//! SUBSCRIBE

use std::convert::From;
use std::error::Error;
use std::fmt;
use std::io::{self, Read, Write};
use std::string::FromUtf8Error;

use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::control::variable_header::PacketIdentifier;
use crate::control::{ControlType, FixedHeader, PacketType};
use crate::encodable::StringEncodeError;
use crate::packet::{Packet, PacketError};
use crate::topic_filter::{TopicFilter, TopicFilterError};
use crate::{Decodable, Encodable, QualityOfService};

/// `SUBSCRIBE` packet
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct SubscribePacket {
    fixed_header: FixedHeader,
    packet_identifier: PacketIdentifier,
    payload: SubscribePacketPayload,
}

impl SubscribePacket {
    pub fn new(pkid: u16, subscribes: Vec<(TopicFilter, QualityOfService)>) -> SubscribePacket {
        let mut pk = SubscribePacket {
            fixed_header: FixedHeader::new(PacketType::with_default(ControlType::Subscribe), 0),
            packet_identifier: PacketIdentifier(pkid),
            payload: SubscribePacketPayload::new(subscribes),
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

impl Packet for SubscribePacket {
    type Payload = SubscribePacketPayload;

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
        let payload: SubscribePacketPayload = SubscribePacketPayload::decode_with(
            reader,
            Some(fixed_header.remaining_length - packet_identifier.encoded_length()),
        )
        .map_err(PacketError::PayloadError)?;
        Ok(SubscribePacket {
            fixed_header,
            packet_identifier,
            payload,
        })
    }
}

/// Payload of subscribe packet
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct SubscribePacketPayload {
    subscribes: Vec<(TopicFilter, QualityOfService)>,
}

impl SubscribePacketPayload {
    pub fn new(subs: Vec<(TopicFilter, QualityOfService)>) -> SubscribePacketPayload {
        SubscribePacketPayload { subscribes: subs }
    }

    pub fn subscribes(&self) -> &[(TopicFilter, QualityOfService)] {
        &self.subscribes[..]
    }
}

impl Encodable for SubscribePacketPayload {
    type Err = SubscribePacketPayloadError;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), Self::Err> {
        for &(ref filter, ref qos) in self.subscribes.iter() {
            filter.encode(writer)?;
            writer.write_u8(*qos as u8)?;
        }

        Ok(())
    }

    fn encoded_length(&self) -> u32 {
        self.subscribes.iter().fold(0, |b, a| b + a.0.encoded_length() + 1)
    }
}

impl Decodable for SubscribePacketPayload {
    type Err = SubscribePacketPayloadError;
    type Cond = u32;

    fn decode_with<R: Read>(
        reader: &mut R,
        payload_len: Option<u32>,
    ) -> Result<SubscribePacketPayload, SubscribePacketPayloadError> {
        let mut payload_len = payload_len.expect("Must provide payload length");
        let mut subs = Vec::new();

        while payload_len > 0 {
            let filter = TopicFilter::decode(reader)?;
            let qos = match reader.read_u8()? {
                0 => QualityOfService::Level0,
                1 => QualityOfService::Level1,
                2 => QualityOfService::Level2,
                _ => return Err(SubscribePacketPayloadError::InvalidQualityOfService),
            };

            payload_len -= filter.encoded_length() + 1;
            subs.push((filter, qos));
        }

        Ok(SubscribePacketPayload::new(subs))
    }
}

#[derive(Debug)]
pub enum SubscribePacketPayloadError {
    IoError(io::Error),
    FromUtf8Error(FromUtf8Error),
    StringEncodeError(StringEncodeError),
    InvalidQualityOfService,
    TopicFilterError(TopicFilterError),
}

impl fmt::Display for SubscribePacketPayloadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SubscribePacketPayloadError::IoError(ref err) => err.fmt(f),
            SubscribePacketPayloadError::FromUtf8Error(ref err) => err.fmt(f),
            SubscribePacketPayloadError::StringEncodeError(ref err) => err.fmt(f),
            SubscribePacketPayloadError::InvalidQualityOfService => write!(f, "Invalid quality of service"),
            SubscribePacketPayloadError::TopicFilterError(ref err) => err.fmt(f),
        }
    }
}

impl Error for SubscribePacketPayloadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            SubscribePacketPayloadError::IoError(ref err) => Some(err),
            SubscribePacketPayloadError::FromUtf8Error(ref err) => Some(err),
            SubscribePacketPayloadError::StringEncodeError(ref err) => Some(err),
            SubscribePacketPayloadError::InvalidQualityOfService => None,
            SubscribePacketPayloadError::TopicFilterError(ref err) => Some(err),
        }
    }
}

impl From<TopicFilterError> for SubscribePacketPayloadError {
    fn from(err: TopicFilterError) -> SubscribePacketPayloadError {
        SubscribePacketPayloadError::TopicFilterError(err)
    }
}

impl From<StringEncodeError> for SubscribePacketPayloadError {
    fn from(err: StringEncodeError) -> SubscribePacketPayloadError {
        SubscribePacketPayloadError::StringEncodeError(err)
    }
}

impl From<io::Error> for SubscribePacketPayloadError {
    fn from(err: io::Error) -> SubscribePacketPayloadError {
        SubscribePacketPayloadError::IoError(err)
    }
}
