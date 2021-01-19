//! Specific packets

use std::fmt;
use std::io::{self, Read, Write};

#[cfg(feature = "tokio")]
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::control::fixed_header::FixedHeaderError;
use crate::control::variable_header::VariableHeaderError;
use crate::control::ControlType;
use crate::control::FixedHeader;
use crate::topic_name::{TopicNameDecodeError, TopicNameError};
use crate::{Decodable, Encodable};

macro_rules! encodable_packet {
    ($typ:ident($($field:ident),* $(,)?)) => {
        impl $crate::encodable::Encodable for $typ {
            fn encode<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                $crate::encodable::Encodable::encode(&self.fixed_header, writer)?;
                $($crate::encodable::Encodable::encode(&self.$field, writer)?;)*
                $crate::encodable::Encodable::encode(&self.payload, writer)?;
                Ok(())
            }

            fn encoded_length(&self) -> u32 {
                $crate::encodable::Encodable::encoded_length(&self.fixed_header)
            }
        }

        impl $crate::packet::EncodablePacket for $typ {}

        impl $typ {
            fn encoded_length_noheader(&self) -> u32 {
                $($crate::encodable::Encodable::encoded_length(&self.$field) +)*
                    $crate::encodable::Encodable::encoded_length(&self.payload)
            }
            #[allow(unused)]
            fn fix_header_remaining_len(&mut self) {
                self.fixed_header.remaining_length = self.encoded_length_noheader()
            }
        }
    };
}

pub use self::connack::ConnackPacket;
pub use self::connect::ConnectPacket;
pub use self::disconnect::DisconnectPacket;
pub use self::pingreq::PingreqPacket;
pub use self::pingresp::PingrespPacket;
pub use self::puback::PubackPacket;
pub use self::pubcomp::PubcompPacket;
pub use self::publish::PublishPacket;
pub use self::pubrec::PubrecPacket;
pub use self::pubrel::PubrelPacket;
pub use self::suback::SubackPacket;
pub use self::subscribe::SubscribePacket;
pub use self::unsuback::UnsubackPacket;
pub use self::unsubscribe::UnsubscribePacket;

pub use self::publish::QoSWithPacketIdentifier;

pub mod connack;
pub mod connect;
pub mod disconnect;
pub mod pingreq;
pub mod pingresp;
pub mod puback;
pub mod pubcomp;
pub mod publish;
pub mod pubrec;
pub mod pubrel;
pub mod suback;
pub mod subscribe;
pub mod unsuback;
pub mod unsubscribe;

/// A trait representing a packet that can be encoded, when passed as `FooPacket` or as
/// `&FooPacket`. Different from [`Encodable`] in that it prevents you from accidentally passing
/// a type intended to be encoded only as a part of a packet and doesn't have a header, e.g.
/// `Vec<u8>`.
pub trait EncodablePacket: Encodable {}

impl<T: EncodablePacket> EncodablePacket for &T {}

/// Methods for encoding and decoding a packet
pub trait Packet: Encodable + fmt::Debug + Sized + 'static {
    type Payload: Encodable + Decodable;

    /// Get the payload
    fn payload(self) -> Self::Payload;
    /// Get a borrow of payload
    fn payload_ref(&self) -> &Self::Payload;

    /// Decode packet given a `FixedHeader`
    fn decode_packet<R: Read>(reader: &mut R, fixed_header: FixedHeader) -> Result<Self, PacketError<Self>>;
}

impl<T: Packet> Decodable for T {
    type Error = PacketError<T>;
    type Cond = Option<FixedHeader>;

    fn decode_with<R: Read>(reader: &mut R, fixed_header: Self::Cond) -> Result<Self, Self::Error> {
        let fixed_header: FixedHeader = if let Some(hdr) = fixed_header {
            hdr
        } else {
            Decodable::decode(reader)?
        };

        <Self as Packet>::decode_packet(reader, fixed_header)
    }
}

/// Parsing errors for packet
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum PacketError<P: Packet> {
    FixedHeaderError(#[from] FixedHeaderError),
    VariableHeaderError(#[from] VariableHeaderError),
    PayloadError(<<P as Packet>::Payload as Decodable>::Error),
    IoError(#[from] io::Error),
    TopicNameError(#[from] TopicNameError),
}

impl<P: Packet> From<TopicNameDecodeError> for PacketError<P> {
    fn from(e: TopicNameDecodeError) -> Self {
        match e {
            TopicNameDecodeError::IoError(e) => e.into(),
            TopicNameDecodeError::InvalidTopicName(e) => e.into(),
        }
    }
}

macro_rules! impl_variable_packet {
    ($($name:ident & $errname:ident => $hdr:ident,)+) => {
        /// Variable packet
        #[derive(Debug, Eq, PartialEq, Clone)]
        pub enum VariablePacket {
            $(
                $name($name),
            )+
        }

        #[cfg(feature = "tokio")]
        impl VariablePacket {
            /// Asynchronously parse a packet from a `tokio::io::AsyncRead`
            ///
            /// This requires mqtt-rs to be built with `feature = "tokio"`
            pub async fn parse<A: AsyncRead + Unpin>(rdr: &mut A) -> Result<Self, VariablePacketError> {
                use std::io::Cursor;
                let fixed_header = FixedHeader::parse(rdr).await?;

                let mut buffer = vec![0u8; fixed_header.remaining_length as usize];
                rdr.read_exact(&mut buffer).await?;

                decode_with_header(&mut Cursor::new(buffer), fixed_header)
            }
        }

        #[inline]
        fn decode_with_header<R: io::Read>(rdr: &mut R, fixed_header: FixedHeader) -> Result<VariablePacket, VariablePacketError> {
            match fixed_header.packet_type.control_type {
                $(
                    ControlType::$hdr => {
                        let pk = <$name as Packet>::decode_packet(rdr, fixed_header)?;
                        Ok(VariablePacket::$name(pk))
                    }
                )+
            }
        }

        $(
            impl From<$name> for VariablePacket {
                fn from(pk: $name) -> VariablePacket {
                    VariablePacket::$name(pk)
                }
            }
        )+

        impl Encodable for VariablePacket {
            fn encode<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
                match *self {
                    $(
                        VariablePacket::$name(ref pk) => pk.encode(writer),
                    )+
                }
            }

            fn encoded_length(&self) -> u32 {
                match *self {
                    $(
                        VariablePacket::$name(ref pk) => pk.encoded_length(),
                    )+
                }
            }
        }

        impl EncodablePacket for VariablePacket {}

        impl Decodable for VariablePacket {
            type Error = VariablePacketError;
            type Cond = Option<FixedHeader>;

            fn decode_with<R: Read>(reader: &mut R, fixed_header: Self::Cond)
                    -> Result<VariablePacket, Self::Error> {
                let fixed_header = match fixed_header {
                    Some(fh) => fh,
                    None => {
                        match FixedHeader::decode(reader) {
                            Ok(header) => header,
                            Err(FixedHeaderError::Unrecognized(code, length)) => {
                                let reader = &mut reader.take(length as u64);
                                let mut buf = Vec::with_capacity(length as usize);
                                reader.read_to_end(&mut buf)?;
                                return Err(VariablePacketError::UnrecognizedPacket(code, buf));
                            },
                            Err(FixedHeaderError::ReservedType(code, length)) => {
                                let reader = &mut reader.take(length as u64);
                                let mut buf = Vec::with_capacity(length as usize);
                                reader.read_to_end(&mut buf)?;
                                return Err(VariablePacketError::ReservedPacket(code, buf));
                            },
                            Err(err) => return Err(From::from(err))
                        }
                    }
                };
                let reader = &mut reader.take(fixed_header.remaining_length as u64);

                decode_with_header(reader, fixed_header)
            }
        }

        /// Parsing errors for variable packet
        #[derive(Debug, thiserror::Error)]
        pub enum VariablePacketError {
            #[error(transparent)]
            FixedHeaderError(#[from] FixedHeaderError),
            #[error("unrecognized packet type ({0}), [u8, ..{}]", .1.len())]
            UnrecognizedPacket(u8, Vec<u8>),
            #[error("reserved packet type ({0}), [u8, ..{}]", .1.len())]
            ReservedPacket(u8, Vec<u8>),
            #[error(transparent)]
            IoError(#[from] io::Error),
            $(
                #[error(transparent)]
                $errname(#[from] PacketError<$name>),
            )+
        }
    }
}

impl_variable_packet! {
    ConnectPacket       & ConnectPacketError        => Connect,
    ConnackPacket       & ConnackPacketError        => ConnectAcknowledgement,

    PublishPacket       & PublishPacketError        => Publish,
    PubackPacket        & PubackPacketError         => PublishAcknowledgement,
    PubrecPacket        & PubrecPacketError         => PublishReceived,
    PubrelPacket        & PubrelPacketError         => PublishRelease,
    PubcompPacket       & PubcompPacketError        => PublishComplete,

    PingreqPacket       & PingreqPacketError        => PingRequest,
    PingrespPacket      & PingrespPacketError       => PingResponse,

    SubscribePacket     & SubscribePacketError      => Subscribe,
    SubackPacket        & SubackPacketError         => SubscribeAcknowledgement,

    UnsubscribePacket   & UnsubscribePacketError    => Unsubscribe,
    UnsubackPacket      & UnsubackPacketError       => UnsubscribeAcknowledgement,

    DisconnectPacket    & DisconnectPacketError     => Disconnect,
}

impl VariablePacket {
    pub fn new<T>(t: T) -> VariablePacket
    where
        VariablePacket: From<T>,
    {
        From::from(t)
    }
}

#[cfg(feature = "tokio-codec")]
mod tokio_codec {
    use super::*;
    use crate::control::packet_type::{PacketType, PacketTypeError};
    use bytes::{Buf, BufMut, BytesMut};
    use tokio_util::codec;

    pub struct MqttDecoder {
        state: DecodeState,
    }

    enum DecodeState {
        Start,
        Packet { length: u32, typ: DecodePacketType },
    }

    #[derive(Copy, Clone)]
    enum DecodePacketType {
        Standard(PacketType),
        Unrecognized(u8),
        Reserved(u8),
    }

    impl MqttDecoder {
        pub const fn new() -> Self {
            MqttDecoder {
                state: DecodeState::Start,
            }
        }
    }

    /// Like FixedHeader::decode(), but on a buffer instead of a stream. Returns None if it reaches
    /// the end of the buffer before it finishes decoding the header.
    #[inline]
    fn decode_header(mut data: &[u8]) -> Option<Result<(DecodePacketType, u32, usize), FixedHeaderError>> {
        let mut header_size = 0;
        macro_rules! read_u8 {
            () => {{
                let (&x, rest) = data.split_first()?;
                data = rest;
                header_size += 1;
                x
            }};
        }

        let type_val = read_u8!();
        let remaining_len = {
            let mut cur = 0u32;
            for i in 0.. {
                let byte = read_u8!();
                cur |= ((byte as u32) & 0x7F) << (7 * i);

                if i >= 4 {
                    return Some(Err(FixedHeaderError::MalformedRemainingLength));
                }

                if byte & 0x80 == 0 {
                    break;
                }
            }

            cur
        };

        let packet_type = match PacketType::from_u8(type_val) {
            Ok(ty) => DecodePacketType::Standard(ty),
            Err(PacketTypeError::UndefinedType(ty, _)) => DecodePacketType::Unrecognized(ty),
            Err(PacketTypeError::ReservedType(ty, _)) => DecodePacketType::Reserved(ty),
            Err(err) => return Some(Err(err.into())),
        };
        Some(Ok((packet_type, remaining_len, header_size)))
    }

    impl codec::Decoder for MqttDecoder {
        type Item = VariablePacket;
        type Error = VariablePacketError;
        fn decode(&mut self, src: &mut BytesMut) -> Result<Option<VariablePacket>, VariablePacketError> {
            loop {
                match &mut self.state {
                    DecodeState::Start => match decode_header(&src[..]) {
                        Some(Ok((typ, length, header_size))) => {
                            src.advance(header_size);
                            self.state = DecodeState::Packet { length, typ };
                            continue;
                        }
                        Some(Err(e)) => return Err(e.into()),
                        None => return Ok(None),
                    },
                    DecodeState::Packet { length, typ } => {
                        let length = *length;
                        if src.remaining() < length as usize {
                            return Ok(None);
                        }
                        let typ = *typ;

                        self.state = DecodeState::Start;

                        match typ {
                            DecodePacketType::Standard(typ) => {
                                let header = FixedHeader {
                                    packet_type: typ,
                                    remaining_length: length,
                                };
                                return decode_with_header(&mut src.reader(), header).map(Some);
                            }
                            DecodePacketType::Unrecognized(code) => {
                                let data = src[..length as usize].to_vec();
                                src.advance(length as usize);
                                return Err(VariablePacketError::UnrecognizedPacket(code, data));
                            }
                            DecodePacketType::Reserved(code) => {
                                let data = src[..length as usize].to_vec();
                                src.advance(length as usize);
                                return Err(VariablePacketError::ReservedPacket(code, data));
                            }
                        }
                    }
                }
            }
        }
    }

    pub struct MqttEncoder {
        _priv: (),
    }

    impl MqttEncoder {
        pub const fn new() -> Self {
            MqttEncoder { _priv: () }
        }
    }

    impl<T: EncodablePacket> codec::Encoder<T> for MqttEncoder {
        type Error = io::Error;
        fn encode(&mut self, packet: T, dst: &mut BytesMut) -> Result<(), io::Error> {
            dst.reserve(packet.encoded_length() as usize);
            packet.encode(&mut dst.writer())
        }
    }

    pub struct MqttCodec {
        decode: MqttDecoder,
        encode: MqttEncoder,
    }

    impl MqttCodec {
        pub const fn new() -> Self {
            MqttCodec {
                decode: MqttDecoder::new(),
                encode: MqttEncoder::new(),
            }
        }
    }

    impl codec::Decoder for MqttCodec {
        type Item = VariablePacket;
        type Error = VariablePacketError;
        #[inline]
        fn decode(&mut self, src: &mut BytesMut) -> Result<Option<VariablePacket>, VariablePacketError> {
            self.decode.decode(src)
        }
    }

    impl<T: EncodablePacket> codec::Encoder<T> for MqttCodec {
        type Error = io::Error;
        #[inline]
        fn encode(&mut self, packet: T, dst: &mut BytesMut) -> Result<(), io::Error> {
            self.encode.encode(packet, dst)
        }
    }
}

#[cfg(feature = "tokio-codec")]
pub use tokio_codec::{MqttCodec, MqttDecoder, MqttEncoder};

#[cfg(test)]
mod test {
    use super::*;

    use std::io::Cursor;

    use crate::{Decodable, Encodable};

    #[test]
    fn test_variable_packet_basic() {
        let packet = ConnectPacket::new("1234".to_owned());

        // Wrap it
        let var_packet = VariablePacket::new(packet);

        // Encode
        let mut buf = Vec::new();
        var_packet.encode(&mut buf).unwrap();

        // Decode
        let mut decode_buf = Cursor::new(buf);
        let decoded_packet = VariablePacket::decode(&mut decode_buf).unwrap();

        assert_eq!(var_packet, decoded_packet);
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_variable_packet_async_parse() {
        let packet = ConnectPacket::new("1234".to_owned());

        // Wrap it
        let var_packet = VariablePacket::new(packet);

        // Encode
        let mut buf = Vec::new();
        var_packet.encode(&mut buf).unwrap();

        // Parse
        let mut async_buf = buf.as_slice();
        let decoded_packet = VariablePacket::parse(&mut async_buf).await.unwrap();

        assert_eq!(var_packet, decoded_packet);
    }

    #[cfg(feature = "tokio-codec")]
    #[tokio::test]
    async fn test_variable_packet_framed() {
        use crate::{QualityOfService, TopicFilter};
        use futures::{SinkExt, StreamExt};
        use tokio_util::codec::{FramedRead, FramedWrite};

        let conn_packet = ConnectPacket::new("1234".to_owned());
        let sub_packet = SubscribePacket::new(1, vec![(TopicFilter::new("foo/#").unwrap(), QualityOfService::Level0)]);

        // small, to make sure buffering and stuff works
        let (reader, writer) = tokio::io::duplex(8);

        let task = tokio::spawn({
            let (conn_packet, sub_packet) = (conn_packet.clone(), sub_packet.clone());
            async move {
                let mut sink = FramedWrite::new(writer, MqttEncodeCodec::new());
                sink.send(conn_packet).await.unwrap();
                sink.send(sub_packet).await.unwrap();
                SinkExt::<VariablePacket>::flush(&mut sink).await.unwrap();
            }
        });

        let mut stream = FramedRead::new(reader, MqttDecodeCodec::new());
        let decoded_conn = stream.next().await.unwrap().unwrap();
        let decoded_sub = stream.next().await.unwrap().unwrap();

        task.await.unwrap();

        assert!(stream.next().await.is_none());

        assert_eq!(decoded_conn, conn_packet.into());
        assert_eq!(decoded_sub, sub_packet.into());
    }
}
