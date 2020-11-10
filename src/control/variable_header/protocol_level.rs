//! Protocol level header

use std::convert::From;
use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::control::variable_header::VariableHeaderError;
use crate::{Decodable, Encodable};

pub const SPEC_3_1_0: u8 = 0x03;
pub const SPEC_3_1_1: u8 = 0x04;
pub const SPEC_5_0: u8 = 0x05;

/// Protocol level in MQTT (`0x04` in v3.1.1)
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum ProtocolLevel {
    Version310 = SPEC_3_1_0,
    Version311 = SPEC_3_1_1,
    Version50 = SPEC_5_0,
}

impl Encodable for ProtocolLevel {
    type Err = VariableHeaderError;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), VariableHeaderError> {
        writer.write_u8(*self as u8).map_err(From::from)
    }

    fn encoded_length(&self) -> u32 {
        1
    }
}

impl Decodable for ProtocolLevel {
    type Err = VariableHeaderError;
    type Cond = ();

    fn decode_with<R: Read>(reader: &mut R, _rest: Option<()>) -> Result<ProtocolLevel, VariableHeaderError> {
        reader
            .read_u8()
            .map_err(From::from)
            .map(ProtocolLevel::from_u8)
            .and_then(|x| x.ok_or(VariableHeaderError::InvalidProtocolVersion))
    }
}

impl ProtocolLevel {
    pub fn from_u8(n: u8) -> Option<ProtocolLevel> {
        match n {
            SPEC_3_1_0 => Some(ProtocolLevel::Version310),
            SPEC_3_1_1 => Some(ProtocolLevel::Version311),
            SPEC_5_0 => Some(ProtocolLevel::Version50),
            _ => None,
        }
    }
}
