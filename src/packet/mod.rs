//! Specific packets

use std::convert::From;
use std::error::Error;
use std::fmt;
use std::io::{self, Cursor, Read, Write};

use futures::Future;
use tokio_io::{io as async_io, AsyncRead};

use control::fixed_header::FixedHeaderError;
use control::variable_header::VariableHeaderError;
use control::ControlType;
use control::FixedHeader;
use encodable::{NoError, StringEncodeError};
use packet::connect::ConnectPacketPayloadError;
use packet::suback::SubackPacketPayloadError;
use packet::subscribe::SubscribePacketPayloadError;
use packet::unsubscribe::UnsubscribePacketPayloadError;
use topic_name::TopicNameError;
use {Decodable, Encodable};

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
    fn encode_variable_headers<W: Write>(&self, writer: &mut W) -> Result<(), PacketError>;
    /// Length of bytes after encoding variable header
    fn encoded_variable_headers_length(&self) -> u32;
    /// Decode packet with a `FixedHeader`
    fn decode_packet<R: Read>(reader: &mut R, fixed_header: FixedHeader) -> Result<Self, PacketError>;
}


impl<T: Packet + fmt::Debug + 'static> Decodable for T {
    type Err = PacketError;
    type Cond = FixedHeader;

    fn decode_with<R: Read>(reader: &mut R, fixed_header: Option<FixedHeader>) -> Result<Self, PacketError> {
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
pub enum PacketError {
    FixedHeaderError(FixedHeaderError),
    VariableHeaderError(VariableHeaderError),
    SubackPacketPayloadError(SubackPacketPayloadError),
    SubscribePacketPayloadError(SubscribePacketPayloadError),
    ConnectPacketPayloadError(ConnectPacketPayloadError),
    UnsubscribePacketPayloadError(UnsubscribePacketPayloadError),
    StringEncodeError(StringEncodeError),
    IoError(io::Error),
    TopicNameError(TopicNameError),
}
impl fmt::Display for PacketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &PacketError::FixedHeaderError(ref err) => err.fmt(f),
            &PacketError::VariableHeaderError(ref err) => err.fmt(f),
            &PacketError::SubackPacketPayloadError(ref err) => err.fmt(f),
            &PacketError::SubscribePacketPayloadError(ref err) => err.fmt(f),
            &PacketError::ConnectPacketPayloadError(ref err) => err.fmt(f),
            &PacketError::UnsubscribePacketPayloadError(ref err) => err.fmt(f),
            &PacketError::StringEncodeError(ref err) => err.fmt(f),
            &PacketError::IoError(ref err) => err.fmt(f),
            &PacketError::TopicNameError(ref err) => err.fmt(f),
        }
    }
}

impl Error for PacketError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            &PacketError::FixedHeaderError(ref err) => Some(err),
            &PacketError::VariableHeaderError(ref err) => Some(err),
            &PacketError::SubackPacketPayloadError(ref err) => Some(err),
            &PacketError::SubscribePacketPayloadError(ref err) => Some(err),
            &PacketError::ConnectPacketPayloadError(ref err) => Some(err),
            &PacketError::UnsubscribePacketPayloadError(ref err) => Some(err),
            &PacketError::StringEncodeError(ref err) => Some(err),
            &PacketError::IoError(ref err) => Some(err),
            &PacketError::TopicNameError(ref err) => Some(err),
        }
    }
}

macro_rules! impl_from_error_maybe_io {
    ($from:ident, $to:ident) => {
        impl From<$from> for $to {
            fn from(err: $from) -> $to {
                match err {
                    $from::IoError(io) => $to::IoError(io),
                    _ => $to::$from(err),
                }
            }
        }
    };
}
impl_from_error_maybe_io!(FixedHeaderError, PacketError);
impl_from_error_maybe_io!(VariableHeaderError, PacketError);
impl_from_error_maybe_io!(StringEncodeError, PacketError);

impl From<io::Error> for PacketError {
    fn from(err: io::Error) -> PacketError {
        PacketError::IoError(err)
    }
}

impl From<NoError> for PacketError {
    fn from(_err: NoError) -> PacketError {
        PacketError::IoError(std::io::Error::new(std::io::ErrorKind::Other, "No Error"))
    }
}

impl From<TopicNameError> for PacketError {
    fn from(err: TopicNameError) -> PacketError {
        match err {
            TopicNameError::StringEncodeError(StringEncodeError::IoError(io)) => PacketError::IoError(io),
            _ => PacketError::TopicNameError(err),
        }
    }
}

impl From<UnsubscribePacketPayloadError> for PacketError {
    fn from(err: UnsubscribePacketPayloadError) -> PacketError {
        match err {
            UnsubscribePacketPayloadError::IoError(io) => PacketError::IoError(io),
            _ => PacketError::UnsubscribePacketPayloadError(err),
        }
    }
}

impl From<SubackPacketPayloadError> for PacketError {
    fn from(err: SubackPacketPayloadError) -> PacketError {
        match err {
            SubackPacketPayloadError::IoError(io) => PacketError::IoError(io),
            _ => PacketError::SubackPacketPayloadError(err),
        }
    }
}

impl From<SubscribePacketPayloadError> for PacketError {
    fn from(err: SubscribePacketPayloadError) -> PacketError {
        match err {
            SubscribePacketPayloadError::IoError(io) => PacketError::IoError(io),
            _ => PacketError::SubscribePacketPayloadError(err),
        }
    }
}

impl From<ConnectPacketPayloadError> for PacketError {
    fn from(err: ConnectPacketPayloadError) -> PacketError {
        match err {
            ConnectPacketPayloadError::IoError(io) => PacketError::IoError(io),
            _ => PacketError::ConnectPacketPayloadError(err),
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

        impl VariablePacket {
            pub fn peek<A: AsyncRead>(rdr: A) -> impl Future<Item = (A, FixedHeader, Vec<u8>), Error = PacketError> {
                FixedHeader::parse(rdr).then(|result| {
                    let (rdr, fixed_header, data) = match result {
                        Ok((rdr, header, data)) => (rdr, header, data),
                        Err(err) => return Err(From::from(err))
                    };

                    Ok((rdr, fixed_header, data))
                })
            }
            pub fn peek_finalize<A: AsyncRead>(rdr: A) -> impl Future<Item = (A, Vec<u8>, Self), Error = PacketError> {
                Self::peek(rdr).and_then(|(rdr, fixed_header, header_buffer)| {
                    let packet = vec![0u8; fixed_header.remaining_length as usize];
                    async_io::read_exact(rdr, packet)
                        .from_err()
                        .and_then(move |(rdr, packet)| {
                            let mut buff_rdr = Cursor::new(packet.clone());
                            let output = match fixed_header.packet_type.control_type {
                                $(
                                    ControlType::$hdr => {
                                        let pk = <$name as Packet>::decode_packet(&mut buff_rdr, fixed_header)?;
                                        VariablePacket::$name(pk)
                                    }
                                )+
                            };
                            let mut result = Vec::new();
                            result.extend(header_buffer);
                            result.extend(packet);
                            Ok((rdr, result, output))
                        })
                })
            }
            pub fn parse<A: AsyncRead>(rdr: A) -> impl Future<Item = (A, Self), Error = PacketError> {
                Self::peek(rdr).and_then(|(rdr, fixed_header, _)| {
                    let buffer = vec![0u8; fixed_header.remaining_length as usize];
                    async_io::read_exact(rdr, buffer)
                        .from_err()
                        .and_then(move |(rdr, buffer)| {
                            let mut buff_rdr = Cursor::new(buffer);
                            let output = match fixed_header.packet_type.control_type {
                                $(
                                    ControlType::$hdr => {
                                        let pk = <$name as Packet>::decode_packet(&mut buff_rdr, fixed_header)?;
                                        VariablePacket::$name(pk)
                                    }
                                )+
                            };

                            Ok((rdr, output))
                        })
                })
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
            type Err = PacketError;

            fn encode<W: Write>(&self, writer: &mut W) -> Result<(), PacketError> {
                match self {
                    $(
                        &VariablePacket::$name(ref pk) => pk.encode(writer).map_err(From::from),
                    )+
                }
            }

            fn encoded_length(&self) -> u32 {
                match self {
                    $(
                        &VariablePacket::$name(ref pk) => pk.encoded_length(),
                    )+
                }
            }
        }
        $(
            impl Encodable for $name {
                type Err = PacketError;

                fn encode<W: Write>(&self, writer: &mut W) -> Result<(), Self::Err> {
                    self.fixed_header().encode(writer)?;
                    self.encode_variable_headers(writer)?;
                    self.payload_ref().encode(writer).map_err(From::from)
                }

                fn encoded_length(&self) -> u32 {
                    self.fixed_header().encoded_length()
                        + self.encoded_variable_headers_length()
                        + self.payload_ref().encoded_length()
                }
            }
        )+

        impl Decodable for VariablePacket {
            type Err = PacketError;
            type Cond = FixedHeader;

            fn decode_with<R: Read>(reader: &mut R, fixed_header: Option<FixedHeader>)
                    -> Result<VariablePacket, Self::Err> {
                let fixed_header = match fixed_header {
                    Some(fh) => fh,
                    None => {
                        match FixedHeader::decode(reader) {
                            Ok(header) => header,
                            // FIXME The "read remaining bytes" behavior seem to not be implemented thoroughly and lacks a unit test
                            // FIXME Dont convert FixedHeaderError to IoError
                            Err(FixedHeaderError::Unrecognized(code, length, mut buf) ) => {
                                let reader = &mut reader.take(length as u64);
                                reader.read_to_end(&mut buf)?;
                                return Err(PacketError::FixedHeaderError(FixedHeaderError::Unrecognized(code, length, buf)));
                            },
                            Err(FixedHeaderError::ReservedType(code, length, mut buf) ) => {
                                let reader = &mut reader.take(length as u64);
                                reader.read_to_end(&mut buf)?;
                                return Err(PacketError::FixedHeaderError(FixedHeaderError::ReservedType(code, length, buf)));
                            },
                            Err(err) => return Err(From::from(err))
                        }
                    }
                };
                let reader = &mut reader.take(fixed_header.remaining_length as u64);

                match fixed_header.packet_type.control_type {
                    $(
                        ControlType::$hdr => {
                            let pk = try!(<$name as Packet>::decode_packet(reader, fixed_header));
                            Ok(VariablePacket::$name(pk))
                        }
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

#[cfg(test)]
mod test {
    use proptest::{collection::vec, num, prelude::*};
    use std::io::{Cursor, ErrorKind};

    use {
        super::*, control::variable_header::ConnectReturnCode, packet::suback::SubscribeReturnCode,
        qos::QualityOfService, Decodable, Encodable, TopicFilter, TopicName,
    };

    #[test]
    fn test_variable_packet_basic() {
        let packet = ConnectPacket::new("MQTT".to_owned(), "1234".to_owned());

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

    #[test]
    fn test_variable_packet_async_parse() {
        use std::io::Cursor;
        let packet = ConnectPacket::new("MQTT".to_owned(), "1234".to_owned());

        // Wrap it
        let var_packet = VariablePacket::new(packet);

        // Encode
        let mut buf = Vec::new();
        var_packet.encode(&mut buf).unwrap();

        // Parse
        let async_buf = Cursor::new(buf);
        match VariablePacket::parse(async_buf).wait() {
            Err(_) => assert!(false),
            Ok((_, decoded_packet)) => assert_eq!(var_packet, decoded_packet),
        }
    }

    #[test]
    fn test_variable_packet_async_peek() {
        use std::io::Cursor;
        let packet = ConnectPacket::new("MQTT".to_owned(), "1234".to_owned());

        // Wrap it
        let var_packet = VariablePacket::new(packet);

        // Encode
        let mut buf = Vec::new();
        var_packet.encode(&mut buf).unwrap();

        // Peek
        let async_buf = Cursor::new(buf.clone());
        match VariablePacket::peek(async_buf.clone()).wait() {
            Err(_) => assert!(false),
            Ok((_, fixed_header, _)) => assert_eq!(fixed_header.packet_type.control_type, ControlType::Connect),
        }

        // Read the rest
        match VariablePacket::peek_finalize(async_buf).wait() {
            Err(_) => assert!(false),
            Ok((_, peeked_buffer, peeked_packet)) => {
                assert_eq!(peeked_buffer, buf);
                assert_eq!(peeked_packet, var_packet);
            }
        }
    }

    prop_compose! {
        fn stg_topicname()(name in "[a-z0-9/]{1,300}") -> TopicName {//FIXME: accept any non-wildcard utf8, enable size 0
            TopicName::new(name).expect("topic")
        }
    }
    prop_compose! {
        fn stg_topicfilter()(name in "[a-z0-9/]{1,300}") -> TopicFilter {//FIXME: accept any non-wildcard utf8
            TopicFilter::new(name).expect("topicfilter")
        }
    }
    prop_compose! {
        fn stg_qos()(level in 0..=2) -> QualityOfService {
            match level {
                0 => QualityOfService::Level0,
                1 => QualityOfService::Level1,
                2 => QualityOfService::Level2,
                _ => unreachable!()
            }
        }
    }
    prop_compose! {
        fn stg_qos_pid()(qos in stg_qos(), pid in num::u16::ANY) -> QoSWithPacketIdentifier {
            QoSWithPacketIdentifier::new(qos, pid)
        }
    }
    prop_compose! {
        fn stg_subretcode()(code in 0..=3) -> SubscribeReturnCode {
            match code {
                0 => SubscribeReturnCode::MaximumQoSLevel0,
                1 => SubscribeReturnCode::MaximumQoSLevel1,
                2 => SubscribeReturnCode::MaximumQoSLevel2,
                3 => SubscribeReturnCode::Failure,
                _ => unreachable!()
            }
        }
    }
    prop_compose! {
        fn stg_connack()(session in proptest::bool::ANY, retcode in num::u8::ANY) -> ConnackPacket {
            ConnackPacket::new(session, ConnectReturnCode::from_u8(retcode))
        }
    }
    prop_compose! {
        fn stg_connect()(prot in "[A-Z]{0,10}", clientid in "[a-z0-9]{1,10}") -> ConnectPacket {
            ConnectPacket::new(prot, clientid)
        }
    }
    prop_compose! {
        fn stg_disconnect()(_ in proptest::bool::ANY) -> DisconnectPacket {
            DisconnectPacket::new()
        }
    }
    prop_compose! {
        fn stg_pingreq()(_ in proptest::bool::ANY) -> PingreqPacket {
            PingreqPacket::new()
        }
    }
    prop_compose! {
        fn stg_pingresp()(_ in proptest::bool::ANY) -> PingrespPacket {
            PingrespPacket::new()
        }
    }
    prop_compose! {
        fn stg_puback()(pkid in num::u16::ANY) -> PubackPacket {
            PubackPacket::new(pkid)
        }
    }
    prop_compose! {
        fn stg_pubcomp()(pkid in num::u16::ANY) -> PubcompPacket {
            PubcompPacket::new(pkid)
        }
    }
    prop_compose! {
        fn stg_publish()(topic in stg_topicname(),
                         qos_pid in stg_qos_pid(),
                         payload in vec(0u8..255u8, 0..300)) -> PublishPacket {
            PublishPacket::new(topic, qos_pid, payload)
        }
    }
    prop_compose! {
        fn stg_pubrec()(pkid in num::u16::ANY) -> PubrecPacket {
            PubrecPacket::new(pkid)
        }
    }
    prop_compose! {
        fn stg_pubrel()(pkid in num::u16::ANY) -> PubrelPacket {
            PubrelPacket::new(pkid)
        }
    }
    prop_compose! {
        fn stg_suback()(pkid in num::u16::ANY, subs in vec(stg_subretcode(), 0..300)) -> SubackPacket {
            SubackPacket::new(pkid, subs)
        }
    }
    prop_compose! {
        fn stg_subscribe()(pkid in num::u16::ANY, subs in vec((stg_topicfilter(),stg_qos()), 0..50)) -> SubscribePacket {
            SubscribePacket::new(pkid, subs)
        }
    }
    prop_compose! {
        fn stg_unsuback()(pkid in num::u16::ANY) -> UnsubackPacket {
            UnsubackPacket::new(pkid)
        }
    }
    prop_compose! {
        fn stg_unsubscribe()(pkid in num::u16::ANY, subs in vec(stg_topicfilter(), 0..50)) -> UnsubscribePacket {
            UnsubscribePacket::new(pkid, subs)
        }
    }
    macro_rules! impl_proptests {
        ($name_rr:ident, $name_eof:ident, $stg:ident, $decodeas:ident, $error:ident) => {
            proptest! {
                /// Encodes packet generated by $stg and checks that `$decodeas::decode()`ing it
                /// yields the original packet back.
                #[test]
                fn $name_rr(pkt in $stg()) {
                    // Encode the packet
                    let mut buf = Vec::new();
                    pkt.encode(&mut buf).expect("encode");

                    // Check that decoding returns the original
                    prop_assert_eq!(pkt, $decodeas::decode(&mut Cursor::new(&buf)).expect("decode full"));
                }
                /// Encodes packet generated by $stg and checks that `$decodeas::decode()`ing it
                /// with some bytes missing yields `$error::IoError(ErrorKind::UnexpectedEof)`.
                #[test]
                fn $name_eof(pkt in $stg()) {
                    // Encode the packet
                    let mut buf = Vec::new();
                    pkt.encode(&mut buf).expect("encode");

                    // Progressively shrink the packet to 0 bytes and try to decode
                    let origlen = buf.len();
                    while buf.pop().is_some() {
                        match $decodeas::decode(&mut Cursor::new(&buf)) {
                            Err($error::IoError(ref i)) if i.kind() == ErrorKind::UnexpectedEof => (),
                            o => prop_assert!(false, "decode {} out of {} bytes => {:?}", buf.len(), origlen, o),
                        }
                    }
                }
            }
        };
    }
    impl_proptests! {roundtrip_connack,     eof_connack,     stg_connack,     ConnackPacket,     PacketError}
    impl_proptests! {roundtrip_connect,     eof_connect,     stg_connect,     ConnectPacket,     PacketError}
    impl_proptests! {roundtrip_disconnect,  eof_disconnect,  stg_disconnect,  DisconnectPacket,  PacketError}
    impl_proptests! {roundtrip_pingreq,     eof_pingreq,     stg_pingreq,     PingreqPacket,     PacketError}
    impl_proptests! {roundtrip_pingresp,    eof_pingresp,    stg_pingresp,    PingrespPacket,    PacketError}
    impl_proptests! {roundtrip_puback,      eof_puback,      stg_puback,      PubackPacket,      PacketError}
    impl_proptests! {roundtrip_pubcomp,     eof_pubcomp,     stg_pubcomp,     PubcompPacket,     PacketError}
    impl_proptests! {roundtrip_publish,     eof_publish,     stg_publish,     PublishPacket,     PacketError}
    impl_proptests! {roundtrip_pubrec,      eof_pubrec,      stg_pubrec,      PubrecPacket,      PacketError}
    impl_proptests! {roundtrip_pubrel,      eof_pubrel,      stg_pubrel,      PubrelPacket,      PacketError}
    impl_proptests! {roundtrip_suback,      eof_suback,      stg_suback,      SubackPacket,      PacketError}
    impl_proptests! {roundtrip_subscribe,   eof_subscribe,   stg_subscribe,   SubscribePacket,   PacketError}
    impl_proptests! {roundtrip_unsuback,    eof_unsuback,    stg_unsuback,    UnsubackPacket,    PacketError}
    impl_proptests! {roundtrip_unsubscribe, eof_unsubscribe, stg_unsubscribe, UnsubscribePacket, PacketError}
}
