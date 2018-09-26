//! Specific packets

use std::convert::From;
use std::error::Error;
use std::fmt;
use std::io::{self, Read, Write, Cursor};

use futures::Future;
use tokio_io::{io as async_io, AsyncRead};

use {Decodable, Encodable};
use control::ControlType;
use control::FixedHeader;
use control::fixed_header::FixedHeaderError;
use control::variable_header::VariableHeaderError;
use encodable::StringEncodeError;
use topic_name::TopicNameError;

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

pub mod connect;
pub mod connack;
pub mod publish;
pub mod puback;
pub mod pubrec;
pub mod pubrel;
pub mod pubcomp;
pub mod pingreq;
pub mod pingresp;
pub mod disconnect;
pub mod subscribe;
pub mod suback;
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

impl<T: Packet + fmt::Debug> Encodable for T {
    type Err = PacketError<T>;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), PacketError<T>> {
        self.fixed_header().encode(writer)?;
        self.encode_variable_headers(writer)?;

        self.payload_ref()
            .encode(writer)
            .map_err(PacketError::PayloadError)
    }

    fn encoded_length(&self) -> u32 {
        self.fixed_header().encoded_length() + self.encoded_variable_headers_length() + self.payload_ref().encoded_length()
    }
}

impl<T: Packet + fmt::Debug> Decodable for T {
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
pub enum PacketError<T: Packet> {
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
        match self {
            &PacketError::FixedHeaderError(ref err) => err.fmt(f),
            &PacketError::VariableHeaderError(ref err) => err.fmt(f),
            &PacketError::PayloadError(ref err) => err.fmt(f),
            &PacketError::MalformedPacket(ref err) => err.fmt(f),
            &PacketError::StringEncodeError(ref err) => err.fmt(f),
            &PacketError::IoError(ref err) => err.fmt(f),
            &PacketError::TopicNameError(ref err) => err.fmt(f),
        }
    }
}

impl<T: Packet + fmt::Debug> Error for PacketError<T> {
    fn description(&self) -> &str {
        match self {
            &PacketError::FixedHeaderError(ref err) => err.description(),
            &PacketError::VariableHeaderError(ref err) => err.description(),
            &PacketError::PayloadError(ref err) => err.description(),
            &PacketError::MalformedPacket(ref err) => &err[..],
            &PacketError::StringEncodeError(ref err) => err.description(),
            &PacketError::IoError(ref err) => err.description(),
            &PacketError::TopicNameError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match self {
            &PacketError::FixedHeaderError(ref err) => Some(err),
            &PacketError::VariableHeaderError(ref err) => Some(err),
            &PacketError::PayloadError(ref err) => Some(err),
            &PacketError::MalformedPacket(..) => None,
            &PacketError::StringEncodeError(ref err) => Some(err),
            &PacketError::IoError(ref err) => Some(err),
            &PacketError::TopicNameError(ref err) => Some(err),
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
        #[derive(Debug, Eq, PartialEq)]
        pub enum VariablePacket {
            $(
                $name($name),
            )+
        }

        impl VariablePacket {
            pub fn peek<A: AsyncRead>(rdr: A) -> impl Future<Item = (A, FixedHeader, Vec<u8>), Error = VariablePacketError> {
                FixedHeader::parse(rdr).then(|result| {
                    let (rdr, fixed_header, data) = match result {
                        Ok((rdr, header, data)) => (rdr, header, data),
                        Err(FixedHeaderError::Unrecognized(code, _length)) => {
                            // can't read excess bytes from rdr as it was dropped when an error
                            // occurred
                            return Err(VariablePacketError::UnrecognizedPacket(code, Vec::new()));
                        },
                        Err(FixedHeaderError::ReservedType(code, _length)) => {
                            // can't read excess bytes from rdr as it was dropped when an error
                            // occurred
                            return Err(VariablePacketError::ReservedPacket(code, Vec::new()));
                        },
                        Err(err) => return Err(From::from(err))
                    };

                    Ok((rdr, fixed_header, data))
                })
            }
            pub fn peek_finalize<A: AsyncRead>(rdr: A) -> impl Future<Item = (A, Vec<u8>, Self), Error = VariablePacketError> {
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
            pub fn parse<A: AsyncRead>(rdr: A) -> impl Future<Item = (A, Self), Error = VariablePacketError> {
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
            type Err = VariablePacketError;

            fn encode<W: Write>(&self, writer: &mut W) -> Result<(), VariablePacketError> {
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
                                try!(reader.read_to_end(&mut buf));
                                return Err(VariablePacketError::UnrecognizedPacket(code, buf));
                            },
                            Err(FixedHeaderError::ReservedType(code, length)) => {
                                let reader = &mut reader.take(length as u64);
                                let mut buf = Vec::with_capacity(length as usize);
                                try!(reader.read_to_end(&mut buf));
                                return Err(VariablePacketError::ReservedPacket(code, buf));
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
                match self {
                    &VariablePacketError::FixedHeaderError(ref err) => err.fmt(f),
                    &VariablePacketError::UnrecognizedPacket(ref code, ref v) =>
                        write!(f, "Unrecognized type ({}), [u8, ..{}]", code, v.len()),
                    &VariablePacketError::ReservedPacket(ref code, ref v) =>
                        write!(f, "Reserved type ({}), [u8, ..{}]", code, v.len()),
                    &VariablePacketError::IoError(ref err) => err.fmt(f),
                    $(
                        &VariablePacketError::$errname(ref err) => err.fmt(f),
                    )+
                }
            }
        }

        impl Error for VariablePacketError {
            fn description(&self) -> &str {
                match self {
                    &VariablePacketError::FixedHeaderError(ref err) => err.description(),
                    &VariablePacketError::UnrecognizedPacket(..) => "Unrecognized packet",
                    &VariablePacketError::ReservedPacket(..) => "Reserved packet",
                    &VariablePacketError::IoError(ref err) => err.description(),
                    $(
                        &VariablePacketError::$errname(ref err) => err.description(),
                    )+
                }
            }

            fn cause(&self) -> Option<&Error> {
                match self {
                    &VariablePacketError::FixedHeaderError(ref err) => Some(err),
                    &VariablePacketError::UnrecognizedPacket(..) => None,
                    &VariablePacketError::ReservedPacket(..) => None,
                    &VariablePacketError::IoError(ref err) => Some(err),
                    $(
                        &VariablePacketError::$errname(ref err) => Some(err),
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
    use super::*;

    use std::io::Cursor;

    use {Decodable, Encodable};

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

}
