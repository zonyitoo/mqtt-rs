use std::convert::From;
use std::io::{Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::control::variable_header::VariableHeaderError;
use crate::{Decodable, Encodable};

/// Packet identifier
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct PacketIdentifier(pub u16);

impl Encodable for PacketIdentifier {
    type Err = VariableHeaderError;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), VariableHeaderError> {
        writer.write_u16::<BigEndian>(self.0).map_err(From::from)
    }

    fn encoded_length(&self) -> u32 {
        2
    }
}

impl Decodable for PacketIdentifier {
    type Err = VariableHeaderError;
    type Cond = ();

    fn decode_with<R: Read>(reader: &mut R, _rest: Option<()>) -> Result<PacketIdentifier, VariableHeaderError> {
        reader.read_u16::<BigEndian>().map(PacketIdentifier).map_err(From::from)
    }
}
