//! UNSUBSCRIBE

use std::io::{self, Read, Write};
use std::string::FromUtf8Error;

use crate::control::variable_header::PacketIdentifier;
use crate::control::{ControlType, FixedHeader, PacketType};
use crate::packet::{DecodablePacket, PacketError};
use crate::topic_filter::{TopicFilter, TopicFilterDecodeError, TopicFilterError};
use crate::{Decodable, Encodable};

/// `UNSUBSCRIBE` packet
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct UnsubscribePacket {
    fixed_header: FixedHeader,
    packet_identifier: PacketIdentifier,
    payload: UnsubscribePacketPayload,
}

encodable_packet!(UnsubscribePacket(packet_identifier, payload));

impl UnsubscribePacket {
    pub fn new(pkid: u16, subscribes: Vec<TopicFilter>) -> UnsubscribePacket {
        let mut pk = UnsubscribePacket {
            fixed_header: FixedHeader::new(PacketType::with_default(ControlType::Unsubscribe), 0),
            packet_identifier: PacketIdentifier(pkid),
            payload: UnsubscribePacketPayload::new(subscribes),
        };
        pk.fix_header_remaining_len();
        pk
    }

    pub fn packet_identifier(&self) -> u16 {
        self.packet_identifier.0
    }

    pub fn set_packet_identifier(&mut self, pkid: u16) {
        self.packet_identifier.0 = pkid;
    }
}

impl DecodablePacket for UnsubscribePacket {
    type Payload = UnsubscribePacketPayload;

    fn decode_packet<R: Read>(reader: &mut R, fixed_header: FixedHeader) -> Result<Self, PacketError<Self>> {
        let packet_identifier: PacketIdentifier = PacketIdentifier::decode(reader)?;
        let payload: UnsubscribePacketPayload = UnsubscribePacketPayload::decode_with(
            reader,
            fixed_header.remaining_length - packet_identifier.encoded_length(),
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
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
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
    type Error = UnsubscribePacketPayloadError;
    type Cond = u32;

    fn decode_with<R: Read>(
        reader: &mut R,
        mut payload_len: u32,
    ) -> Result<UnsubscribePacketPayload, UnsubscribePacketPayloadError> {
        let mut subs = Vec::new();

        while payload_len > 0 {
            let filter = TopicFilter::decode(reader)?;
            payload_len -= filter.encoded_length();
            subs.push(filter);
        }

        Ok(UnsubscribePacketPayload::new(subs))
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum UnsubscribePacketPayloadError {
    IoError(#[from] io::Error),
    FromUtf8Error(#[from] FromUtf8Error),
    TopicFilterError(#[from] TopicFilterError),
}

impl From<TopicFilterDecodeError> for UnsubscribePacketPayloadError {
    fn from(e: TopicFilterDecodeError) -> Self {
        match e {
            TopicFilterDecodeError::IoError(e) => e.into(),
            TopicFilterDecodeError::InvalidTopicFilter(e) => e.into(),
        }
    }
}
