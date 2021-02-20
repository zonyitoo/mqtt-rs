//! SUBSCRIBE

use std::io::{self, Read, Write};
use std::string::FromUtf8Error;

use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::control::variable_header::PacketIdentifier;
use crate::control::{ControlType, FixedHeader, PacketType};
use crate::packet::{DecodablePacket, PacketError};
use crate::topic_filter::{TopicFilter, TopicFilterDecodeError, TopicFilterError};
use crate::{Decodable, Encodable, QualityOfService};

/// `SUBSCRIBE` packet
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct SubscribePacket {
    fixed_header: FixedHeader,
    packet_identifier: PacketIdentifier,
    payload: SubscribePacketPayload,
}

encodable_packet!(SubscribePacket(packet_identifier, payload));

impl SubscribePacket {
    pub fn new(pkid: u16, subscribes: Vec<(TopicFilter, QualityOfService)>) -> SubscribePacket {
        let mut pk = SubscribePacket {
            fixed_header: FixedHeader::new(PacketType::with_default(ControlType::Subscribe), 0),
            packet_identifier: PacketIdentifier(pkid),
            payload: SubscribePacketPayload::new(subscribes),
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

impl DecodablePacket for SubscribePacket {
    type Payload = SubscribePacketPayload;

    fn decode_packet<R: Read>(reader: &mut R, fixed_header: FixedHeader) -> Result<Self, PacketError<Self>> {
        let packet_identifier: PacketIdentifier = PacketIdentifier::decode(reader)?;
        let payload: SubscribePacketPayload = SubscribePacketPayload::decode_with(
            reader,
            fixed_header.remaining_length - packet_identifier.encoded_length(),
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
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
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
    type Error = SubscribePacketPayloadError;
    type Cond = u32;

    fn decode_with<R: Read>(
        reader: &mut R,
        mut payload_len: u32,
    ) -> Result<SubscribePacketPayload, SubscribePacketPayloadError> {
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

#[derive(Debug, thiserror::Error)]
pub enum SubscribePacketPayloadError {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),
    #[error("invalid quality of service")]
    InvalidQualityOfService,
    #[error(transparent)]
    TopicFilterError(#[from] TopicFilterError),
}

impl From<TopicFilterDecodeError> for SubscribePacketPayloadError {
    fn from(e: TopicFilterDecodeError) -> Self {
        match e {
            TopicFilterDecodeError::IoError(e) => e.into(),
            TopicFilterDecodeError::InvalidTopicFilter(e) => e.into(),
        }
    }
}
