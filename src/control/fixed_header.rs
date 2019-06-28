//! Fixed header in MQTT

use std::convert::From;
use std::error::Error;
use std::fmt;
use std::io::{self, Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt};

use futures::{future, Future};
use tokio_io::{io as async_io, AsyncRead};

use control::packet_type::{ControlType, PacketType, PacketTypeError};
use {Decodable, Encodable};

/// Fixed header for each MQTT control packet
///
/// Format:
///
/// ```plain
/// 7                          3                          0
/// +--------------------------+--------------------------+
/// | MQTT Control Packet Type | Flags for each type      |
/// +--------------------------+--------------------------+
/// | Remaining Length ...                                |
/// +-----------------------------------------------------+
/// ```
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct FixedHeader {
    /// Packet Type
    pub packet_type: PacketType,

    /// The Remaining Length is the number of bytes remaining within the current packet,
    /// including data in the variable header and the payload. The Remaining Length does
    /// not include the bytes used to encode the Remaining Length.
    pub remaining_length: u32,
}

impl FixedHeader {
    pub fn new(packet_type: PacketType, remaining_length: u32) -> FixedHeader {
        debug_assert!(remaining_length <= 0x0FFFFFFF);
        FixedHeader {
            packet_type: packet_type,
            remaining_length: remaining_length,
        }
    }

    /// Asynchronously parse a single fixed header from an AsyncRead type, such as a network
    /// socket.
    pub fn parse<A: AsyncRead>(rdr: A) -> impl Future<Item = (A, Self, Vec<u8>), Error = FixedHeaderError> {
        async_io::read_exact(rdr, [0u8])
            .from_err()
            .and_then(|(rdr, [type_val])| {
                let mut data: Vec<u8> = Vec::new();
                data.push(type_val);
                future::loop_fn((rdr, 0, 0, data), move |(rdr, mut cur, i, mut data)| {
                    async_io::read_exact(rdr, [0u8])
                        .from_err()
                        .and_then(move |(rdr, [byte])| {
                            data.push(byte);

                            cur |= (u32::from(byte) & 0x7F) << (7 * i);

                            if i >= 4 {
                                return Err(FixedHeaderError::MalformedRemainingLength);
                            }

                            if byte & 0x80 == 0 {
                                Ok(future::Loop::Break((rdr, cur, data)))
                            } else {
                                Ok(future::Loop::Continue((rdr, cur, i + 1, data)))
                            }
                        })
                })
                .and_then(move |(rdr, len, data)| match PacketType::from_u8(type_val) {
                    Ok(packet_type) => Ok((rdr, FixedHeader::new(packet_type, len), data)),
                    Err(PacketTypeError::ReservedType(ty, fl)) => Err(FixedHeaderError::ReservedType(ty, fl, len)),
                    Err(PacketTypeError::InvalidFlag(ty, fl)) => Err(FixedHeaderError::InvalidFlag(ty, fl, len)),
                })
            })
    }
}

impl Encodable for FixedHeader {
    type Err = FixedHeaderError;

    fn encode<W: Write>(&self, wr: &mut W) -> Result<(), FixedHeaderError> {
        wr.write_u8(self.packet_type.to_u8())?;

        let mut cur_len = self.remaining_length;
        loop {
            let mut byte = (cur_len & 0x7F) as u8;
            cur_len >>= 7;

            if cur_len > 0 {
                byte |= 0x80;
            }

            wr.write_u8(byte)?;

            if cur_len == 0 {
                break;
            }
        }

        Ok(())
    }

    fn encoded_length(&self) -> u32 {
        let rem_size = if self.remaining_length >= 2_097_152 {
            4
        } else if self.remaining_length >= 16_384 {
            3
        } else if self.remaining_length >= 128 {
            2
        } else {
            1
        };
        1 + rem_size
    }
}

impl Decodable for FixedHeader {
    type Err = FixedHeaderError;
    type Cond = ();

    fn decode_with<R: Read>(rdr: &mut R, _rest: Option<()>) -> Result<FixedHeader, FixedHeaderError> {
        let type_val = rdr.read_u8()?;
        let len = {
            let mut cur = 0u32;
            for i in 0.. {
                let byte = rdr.read_u8()?;
                cur |= ((byte as u32) & 0x7F) << (7 * i);

                if i >= 4 {
                    return Err(FixedHeaderError::MalformedRemainingLength);
                }

                if byte & 0x80 == 0 {
                    break;
                }
            }

            cur
        };

        match PacketType::from_u8(type_val) {
            Ok(packet_type) => Ok(FixedHeader::new(packet_type, len)),
            Err(PacketTypeError::ReservedType(ty, fl)) => Err(FixedHeaderError::ReservedType(ty, fl, len)),
            Err(PacketTypeError::InvalidFlag(ty, fl)) => Err(FixedHeaderError::InvalidFlag(ty, fl, len)),
        }
    }
}

/// Error while decoding the "Fixed Header" (MQTT [section 2.2]).
///
/// The spec mandates that you close the connection in this case ([section 4.8]), but there's enough
/// information here (parsed type, flag, lenght) should you want to read and parse the rest of the
/// packet manually.
///
/// [section 2.2]: http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/errata01/os/mqtt-v3.1.1-errata01-os-complete.html#_Toc442180833
/// [section 4.8]: http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/errata01/os/mqtt-v3.1.1-errata01-os-complete.html#_Toc442180923
#[derive(Debug)]
pub enum FixedHeaderError {
    IoError(io::Error),
    /// Illegal remaining length value. In practice meaning the 4th byte's msb is set.
    MalformedRemainingLength,
    /// Packet types 0 and 15 are reserved by spec.
    ReservedType(u8, u8, u32),
    /// Invalid flag for this packet type. In practice, only PUBLISH packets can have varying flags.
    InvalidFlag(ControlType, u8, u32),
}

impl From<io::Error> for FixedHeaderError {
    fn from(err: io::Error) -> FixedHeaderError {
        FixedHeaderError::IoError(err)
    }
}

impl fmt::Display for FixedHeaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &FixedHeaderError::IoError(ref err) => write!(f, "{}", err),
            &FixedHeaderError::MalformedRemainingLength => write!(f, "Malformed remaining length"),
            &FixedHeaderError::ReservedType(ty, fl, len) => write!(f, "Reserved type ({}, {}, {})", ty, fl, len),
            &FixedHeaderError::InvalidFlag(ty, fl, len) => write!(f, "Invalid flag ({:?}, {}, {})", ty, fl, len),
        }
    }
}

impl Error for FixedHeaderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            &FixedHeaderError::IoError(ref err) => Some(err),
            &FixedHeaderError::MalformedRemainingLength => None,
            &FixedHeaderError::ReservedType(..) => None,
            &FixedHeaderError::InvalidFlag(..) => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use control::packet_type::{ControlType, PacketType};
    use std::io::Cursor;
    use {Decodable, Encodable};

    #[test]
    fn test_encode_fixed_header() {
        let header = FixedHeader::new(PacketType::with_default(ControlType::Connect), 321);
        let mut buf = Vec::new();
        header.encode(&mut buf).unwrap();

        let expected = b"\x10\xc1\x02";
        assert_eq!(&expected[..], &buf[..]);
    }

    #[test]
    fn test_decode_fixed_header() {
        let stream = b"\x10\xc1\x02";
        let mut cursor = Cursor::new(&stream[..]);
        let header = FixedHeader::decode(&mut cursor).unwrap();
        assert_eq!(header.packet_type, PacketType::with_default(ControlType::Connect));
        assert_eq!(header.remaining_length, 321);
    }

    #[test]
    #[should_panic]
    fn test_decode_too_long_fixed_header() {
        let stream = b"\x10\x80\x80\x80\x80\x02";
        let mut cursor = Cursor::new(&stream[..]);
        FixedHeader::decode(&mut cursor).unwrap();
    }
}
