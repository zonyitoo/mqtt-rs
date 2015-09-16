use std::io::{Read, Write};
use std::error::Error;
use std::fmt;
use std::convert::From;

use control::{FixedHeader, VariableHeader};
use control::fixed_header::FixedHeaderError;
use control::variable_header::VariableHeaderError;
use control::ControlType;
use {Encodable, Decodable};

pub use self::connect::ConnectPacket;
pub use self::connack::ConnackPacket;

pub mod connect;
pub mod connack;

pub trait Packet<'a> {
    type Payload: Encodable<'a> + Decodable<'a> + 'a;

    fn fixed_header(&self) -> &FixedHeader;
    fn variable_headers(&self) -> &[VariableHeader];
    fn payload(&self) -> &Self::Payload;

    fn decode_packet<R: Read>(reader: &mut R, fixed_header: FixedHeader) -> Result<Self, PacketError<'a, Self>>;
}

impl<'a, T: Packet<'a> + fmt::Debug + 'a> Encodable<'a> for T {
    type Err = PacketError<'a, T>;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), PacketError<'a, T>> {
        try!(self.fixed_header().encode(writer));
        for varheader in self.variable_headers() {
            try!(varheader.encode(writer));
        }

        self.payload().encode(writer).map_err(PacketError::PayloadError)
    }

    fn encoded_length(&self) -> u32 {
        self.fixed_header().encoded_length()
            + self.variable_headers().iter().fold(0, |b, a| b + a.encoded_length())
            + self.payload().encoded_length()
    }
}

impl<'a, T: Packet<'a> + fmt::Debug + 'a> Decodable<'a> for T {
    type Err = PacketError<'a, T>;
    type Cond = FixedHeader;

    fn decode_with<R: Read>(reader: &mut R, fixed_header: Option<FixedHeader>)
            -> Result<Self, PacketError<'a, Self>> {
        let fixed_header: FixedHeader =
            if let Some(hdr) = fixed_header {
                hdr
            } else {
                try!(Decodable::decode(reader))
            };

        <Self as Packet>::decode_packet(reader, fixed_header)
    }
}

#[derive(Debug)]
pub enum PacketError<'a, T: Packet<'a>> {
    FixedHeaderError(FixedHeaderError),
    VariableHeaderError(VariableHeaderError),
    PayloadError(<<T as Packet<'a>>::Payload as Encodable<'a>>::Err),
}

impl<'a, T: Packet<'a>> fmt::Display for PacketError<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &PacketError::FixedHeaderError(ref err) => err.fmt(f),
            &PacketError::VariableHeaderError(ref err) => err.fmt(f),
            &PacketError::PayloadError(ref err) => err.fmt(f),
        }
    }
}

impl<'a, T: Packet<'a> + fmt::Debug> Error for PacketError<'a, T> {
    fn description(&self) -> &str {
        match self {
            &PacketError::FixedHeaderError(ref err) => err.description(),
            &PacketError::VariableHeaderError(ref err) => err.description(),
            &PacketError::PayloadError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match self {
            &PacketError::FixedHeaderError(ref err) => err.cause(),
            &PacketError::VariableHeaderError(ref err) => err.cause(),
            &PacketError::PayloadError(ref err) => err.cause(),
        }
    }
}

impl<'a, T: Packet<'a>> From<FixedHeaderError> for PacketError<'a, T> {
    fn from(err: FixedHeaderError) -> PacketError<'a, T> {
        PacketError::FixedHeaderError(err)
    }
}

impl<'a, T: Packet<'a>> From<VariableHeaderError> for PacketError<'a, T> {
    fn from(err: VariableHeaderError) -> PacketError<'a, T> {
        PacketError::VariableHeaderError(err)
    }
}

macro_rules! impl_variable_packet {
    ($($name:ident & $errname:ident => $hdr:ident,)+) => {
        #[derive(Debug, Eq, PartialEq)]
        pub enum VariablePacket {
            $(
                $name($name),
            )+
        }

        $(
            impl From<$name> for VariablePacket {
                fn from(pk: $name) -> VariablePacket {
                    VariablePacket::$name(pk)
                }
            }
        )+

        impl<'a> Encodable<'a> for VariablePacket {
            type Err = VariablePacketError<'a>;

            fn encode<W: Write>(&self, writer: &mut W) -> Result<(), VariablePacketError<'a>> {
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

        impl<'a> Decodable<'a> for VariablePacket {
            type Err = VariablePacketError<'a>;
            type Cond = FixedHeader;

            fn decode_with<R: Read>(reader: &mut R, fixed_header: Option<FixedHeader>)
                    -> Result<VariablePacket, Self::Err> {
                let fixed_header = match fixed_header {
                    Some(fh) => fh,
                    None => try!(FixedHeader::decode(reader)),
                };

                match fixed_header.packet_type.control_type() {
                    $(
                        ControlType::$hdr => {
                            let pk = try!(<$name as Packet<'a>>::decode_packet(reader, fixed_header));
                            Ok(VariablePacket::$name(pk))
                        }
                    )+

                    _ => return Err(VariablePacketError::UnrecognizedFixedHeader(fixed_header)),
                }
            }
        }

        #[derive(Debug)]
        pub enum VariablePacketError<'a> {
            FixedHeaderError(FixedHeaderError),
            UnrecognizedFixedHeader(FixedHeader),
            $(
                $errname(PacketError<'a, $name>),
            )+
        }

        impl<'a> From<FixedHeaderError> for VariablePacketError<'a> {
            fn from(err: FixedHeaderError) -> VariablePacketError<'a> {
                VariablePacketError::FixedHeaderError(err)
            }
        }

        $(
            impl<'a> From<PacketError<'a, $name>> for VariablePacketError<'a> {
                fn from(err: PacketError<'a, $name>) -> VariablePacketError<'a> {
                    VariablePacketError::$errname(err)
                }
            }
        )+

        impl<'a> fmt::Display for VariablePacketError<'a> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    &VariablePacketError::FixedHeaderError(ref err) => err.fmt(f),
                    &VariablePacketError::UnrecognizedFixedHeader(..) => write!(f, "Unrecognized fixed header"),
                    $(
                        &VariablePacketError::$errname(ref err) => err.fmt(f),
                    )+
                }
            }
        }

        impl<'a> Error for VariablePacketError<'a> {
            fn description(&self) -> &str {
                match self {
                    &VariablePacketError::FixedHeaderError(ref err) => err.description(),
                    &VariablePacketError::UnrecognizedFixedHeader(..) => "Unrecognized fixed header",
                    $(
                        &VariablePacketError::$errname(ref err) => err.description(),
                    )+
                }
            }

            fn cause(&self) -> Option<&Error> {
                match self {
                    &VariablePacketError::FixedHeaderError(ref err) => Some(err),
                    &VariablePacketError::UnrecognizedFixedHeader(..) => None,
                    $(
                        &VariablePacketError::$errname(ref err) => Some(err),
                    )+
                }
            }
        }
    }
}

impl_variable_packet! {
    ConnectPacket & ConnectPacketError => Connect,
    ConnackPacket & ConnackPacketError => ConnectAcknowledgement,
}

impl VariablePacket {
    pub fn new<T>(t: T) -> VariablePacket
        where VariablePacket: From<T>
    {
        From::from(t)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::io::Cursor;

    use {Encodable, Decodable};

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
}
