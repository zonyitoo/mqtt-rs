//! Specific packets

use std::convert::From;
use std::error::Error;
use std::fmt;
use std::io::{self, Read, Write};

#[cfg(feature = "async")]
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::control::fixed_header::FixedHeaderError;
use crate::control::variable_header::VariableHeaderError;
use crate::control::ControlType;
use crate::control::FixedHeader;
use crate::encodable::StringEncodeError;
use crate::topic_name::TopicNameError;
use crate::{Decodable, Encodable};

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

/// Methods for encoding and decoding a packet
pub trait Packet: Sized {
    type Payload: Encodable + Decodable;

    /// Get a `FixedHeader` of this packet
    fn fixed_header(&self) -> &FixedHeader;
    /// Get the payload
    fn payload(self) -> Self::Payload;
    /// Get a borrow of payload
    fn payload_ref(&self) -> &Self::Payload;

    /// Encode variable headers to writer
    fn encode_variable_headers<W: Write>(&self, writer: &mut W) -> Result<(), PacketError<Self>>;
    /// Length of bytes after encoding variable header
    fn encoded_variable_headers_length(&self) -> u32;
    /// Deocde packet with a `FixedHeader`
    fn decode_packet<R: Read>(reader: &mut R, fixed_header: FixedHeader) -> Result<Self, PacketError<Self>>;
}

impl<T: Packet + fmt::Debug + 'static> Encodable for T {
    type Err = PacketError<T>;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), PacketError<T>> {
        self.fixed_header().encode(writer)?;
        self.encode_variable_headers(writer)?;

        self.payload_ref().encode(writer).map_err(PacketError::PayloadError)
    }

    fn encoded_length(&self) -> u32 {
        self.fixed_header().encoded_length()
            + self.encoded_variable_headers_length()
            + self.payload_ref().encoded_length()
    }
}

impl<T: Packet + fmt::Debug + 'static> Decodable for T {
    type Err = PacketError<T>;
    type Cond = FixedHeader;

    fn decode_with<R: Read>(reader: &mut R, fixed_header: Option<FixedHeader>) -> Result<Self, PacketError<Self>> {
        let fixed_header: FixedHeader = if let Some(hdr) = fixed_header {
            hdr
        } else {
            Decodable::decode(reader)?
        };

        <Self as Packet>::decode_packet(reader, fixed_header)
    }
}

/// Parsing errors for packet
#[derive(Debug)]
pub enum PacketError<T: Packet + 'static> {
    FixedHeaderError(FixedHeaderError),
    VariableHeaderError(VariableHeaderError),
    PayloadError(<<T as Packet>::Payload as Encodable>::Err),
    MalformedPacket(String),
    StringEncodeError(StringEncodeError),
    IoError(io::Error),
    TopicNameError(TopicNameError),
}

impl<T: Packet> fmt::Display for PacketError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PacketError::FixedHeaderError(ref err) => err.fmt(f),
            PacketError::VariableHeaderError(ref err) => err.fmt(f),
            PacketError::PayloadError(ref err) => err.fmt(f),
            PacketError::MalformedPacket(ref err) => err.fmt(f),
            PacketError::StringEncodeError(ref err) => err.fmt(f),
            PacketError::IoError(ref err) => err.fmt(f),
            PacketError::TopicNameError(ref err) => err.fmt(f),
        }
    }
}

impl<T: Packet + fmt::Debug> Error for PacketError<T> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            PacketError::FixedHeaderError(ref err) => Some(err),
            PacketError::VariableHeaderError(ref err) => Some(err),
            PacketError::PayloadError(ref err) => Some(err),
            PacketError::MalformedPacket(..) => None,
            PacketError::StringEncodeError(ref err) => Some(err),
            PacketError::IoError(ref err) => Some(err),
            PacketError::TopicNameError(ref err) => Some(err),
        }
    }
}

impl<T: Packet> From<FixedHeaderError> for PacketError<T> {
    fn from(err: FixedHeaderError) -> PacketError<T> {
        PacketError::FixedHeaderError(err)
    }
}

impl<T: Packet> From<VariableHeaderError> for PacketError<T> {
    fn from(err: VariableHeaderError) -> PacketError<T> {
        PacketError::VariableHeaderError(err)
    }
}

impl<T: Packet> From<io::Error> for PacketError<T> {
    fn from(err: io::Error) -> PacketError<T> {
        PacketError::IoError(err)
    }
}

impl<T: Packet> From<StringEncodeError> for PacketError<T> {
    fn from(err: StringEncodeError) -> PacketError<T> {
        PacketError::StringEncodeError(err)
    }
}

impl<T: Packet> From<TopicNameError> for PacketError<T> {
    fn from(err: TopicNameError) -> PacketError<T> {
        PacketError::TopicNameError(err)
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

        #[cfg(feature = "async")]
        impl VariablePacket {
            /// Asynchronously parse a packet from a `tokio::io::AsyncRead`
            ///
            /// This requires mqtt-rs to be built with `feature = "async"`
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
            type Err = VariablePacketError;

            fn encode<W: Write>(&self, writer: &mut W) -> Result<(), VariablePacketError> {
                match *self {
                    $(
                        VariablePacket::$name(ref pk) => pk.encode(writer).map_err(From::from),
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

        impl Decodable for VariablePacket {
            type Err = VariablePacketError;
            type Cond = FixedHeader;

            fn decode_with<R: Read>(reader: &mut R, fixed_header: Option<FixedHeader>)
                    -> Result<VariablePacket, Self::Err> {
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
        #[derive(Debug)]
        pub enum VariablePacketError {
            FixedHeaderError(FixedHeaderError),
            UnrecognizedPacket(u8, Vec<u8>),
            ReservedPacket(u8, Vec<u8>),
            IoError(io::Error),
            $(
                $errname(PacketError<$name>),
            )+
        }

        impl From<FixedHeaderError> for VariablePacketError {
            fn from(err: FixedHeaderError) -> VariablePacketError {
                VariablePacketError::FixedHeaderError(err)
            }
        }

        impl From<io::Error> for VariablePacketError {
            fn from(err: io::Error) -> VariablePacketError {
                VariablePacketError::IoError(err)
            }
        }

        $(
            impl From<PacketError<$name>> for VariablePacketError {
                fn from(err: PacketError<$name>) -> VariablePacketError {
                    VariablePacketError::$errname(err)
                }
            }
        )+

        impl fmt::Display for VariablePacketError {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match *self {
                    VariablePacketError::FixedHeaderError(ref err) => err.fmt(f),
                    VariablePacketError::UnrecognizedPacket(ref code, ref v) =>
                        write!(f, "Unrecognized type ({}), [u8, ..{}]", code, v.len()),
                    VariablePacketError::ReservedPacket(ref code, ref v) =>
                        write!(f, "Reserved type ({}), [u8, ..{}]", code, v.len()),
                    VariablePacketError::IoError(ref err) => err.fmt(f),
                    $(
                        VariablePacketError::$errname(ref err) => err.fmt(f),
                    )+
                }
            }
        }

        impl Error for VariablePacketError {
            fn source(&self) -> Option<&(dyn Error + 'static)> {
                match *self {
                    VariablePacketError::FixedHeaderError(ref err) => Some(err),
                    VariablePacketError::UnrecognizedPacket(..) => None,
                    VariablePacketError::ReservedPacket(..) => None,
                    VariablePacketError::IoError(ref err) => Some(err),
                    $(
                        VariablePacketError::$errname(ref err) => Some(err),
                    )+
                }
            }
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
    use crate::control::PacketType;
    use bytes::{Buf, BufMut, BytesMut};
    use tokio_util::codec;

    pub struct MqttDecodeCodec {
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

    impl MqttDecodeCodec {
        pub const fn new() -> Self {
            MqttDecodeCodec {
                state: DecodeState::Start,
            }
        }
    }

    impl codec::Decoder for MqttDecodeCodec {
        type Item = VariablePacket;
        type Error = VariablePacketError;
        fn decode(&mut self, src: &mut BytesMut) -> Result<Option<VariablePacket>, VariablePacketError> {
            loop {
                match &mut self.state {
                    DecodeState::Start => {
                        let mut slice = &src[..];
                        let start_len = slice.len();
                        let (typ, length) = match FixedHeader::decode(&mut slice) {
                            Ok(header) => (DecodePacketType::Standard(header.packet_type), header.remaining_length),
                            Err(FixedHeaderError::Unrecognized(code, length)) => {
                                (DecodePacketType::Unrecognized(code), length)
                            }
                            Err(FixedHeaderError::ReservedType(code, length)) => {
                                (DecodePacketType::Reserved(code), length)
                            }
                            Err(FixedHeaderError::IoError(e)) if e.kind() == io::ErrorKind::UnexpectedEof => {
                                return Ok(None)
                            }
                            Err(e) => return Err(e.into()),
                        };
                        let header_size = start_len - slice.len();
                        src.advance(header_size);
                        self.state = DecodeState::Packet { length, typ };
                        continue;
                    }
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

    pub struct MqttEncodeCodec {
        _priv: (),
    }

    impl MqttEncodeCodec {
        pub const fn new() -> Self {
            MqttEncodeCodec { _priv: () }
        }
    }

    pub trait MqttEncodablePacket {
        type Error: From<io::Error>;
        fn encoded_length(&self) -> u32;
        fn encode<W: io::Write>(&self, writer: &mut W) -> Result<(), Self::Error>;
    }
    impl<T: Packet + fmt::Debug + 'static> MqttEncodablePacket for T {
        type Error = PacketError<T>;
        #[inline]
        fn encoded_length(&self) -> u32 {
            Encodable::encoded_length(self)
        }
        #[inline]
        fn encode<W: io::Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
            Encodable::encode(self, writer)
        }
    }
    impl MqttEncodablePacket for VariablePacket {
        type Error = VariablePacketError;
        #[inline]
        fn encoded_length(&self) -> u32 {
            Encodable::encoded_length(self)
        }
        #[inline]
        fn encode<W: io::Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
            Encodable::encode(self, writer)
        }
    }

    impl<T: MqttEncodablePacket> codec::Encoder<T> for MqttEncodeCodec {
        type Error = T::Error;
        fn encode(&mut self, packet: T, dst: &mut BytesMut) -> Result<(), T::Error> {
            dst.reserve(packet.encoded_length() as usize);
            packet.encode(&mut dst.writer())
        }
    }

    pub struct MqttCodec {
        decode: MqttDecodeCodec,
        encode: MqttEncodeCodec,
    }

    impl MqttCodec {
        pub const fn new() -> Self {
            MqttCodec {
                decode: MqttDecodeCodec::new(),
                encode: MqttEncodeCodec::new(),
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

    impl<T: MqttEncodablePacket> codec::Encoder<T> for MqttCodec {
        type Error = T::Error;
        #[inline]
        fn encode(&mut self, packet: T, dst: &mut BytesMut) -> Result<(), T::Error> {
            self.encode.encode(packet, dst)
        }
    }
}

#[cfg(feature = "tokio-codec")]
pub use tokio_codec::{MqttCodec, MqttDecodeCodec, MqttEncodeCodec};

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

    #[cfg(feature = "async")]
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
