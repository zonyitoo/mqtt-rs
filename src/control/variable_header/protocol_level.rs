//! Protocol level header

use std::convert::From;
use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt};

use control::variable_header::VariableHeaderError;
use {Decodable, Encodable};

pub const SPEC_3_1_1: u8 = 0x04;

/// Protocol level in MQTT (`0x04` in v3.1.1)
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct ProtocolLevel(pub u8);

impl Encodable for ProtocolLevel {
    type Err = VariableHeaderError;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), VariableHeaderError> {
        writer.write_u8(self.0).map_err(From::from)
    }

    fn encoded_length(&self) -> u32 {
        1
    }
}

impl Decodable for ProtocolLevel {
    type Err = VariableHeaderError;
    type Cond = ();

    fn decode_with<R: Read>(reader: &mut R, _rest: Option<()>) -> Result<ProtocolLevel, VariableHeaderError> {
        reader.read_u8().map(ProtocolLevel).map_err(From::from)
    }
}
