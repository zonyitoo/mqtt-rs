use std::convert::From;
use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::control::variable_header::VariableHeaderError;
use crate::{Decodable, Encodable};

/// Flags in `CONNACK` packet
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct ConnackFlags {
    pub session_present: bool,
}

impl ConnackFlags {
    pub fn empty() -> ConnackFlags {
        ConnackFlags { session_present: false }
    }
}

impl Encodable for ConnackFlags {
    type Err = VariableHeaderError;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), VariableHeaderError> {
        let code = self.session_present as u8;
        writer.write_u8(code).map_err(From::from)
    }

    fn encoded_length(&self) -> u32 {
        1
    }
}

impl Decodable for ConnackFlags {
    type Err = VariableHeaderError;
    type Cond = ();

    fn decode_with<R: Read>(reader: &mut R, _rest: Option<()>) -> Result<ConnackFlags, VariableHeaderError> {
        let code = reader.read_u8()?;
        if code & !1 != 0 {
            return Err(VariableHeaderError::InvalidReservedFlag);
        }

        Ok(ConnackFlags {
            session_present: code == 1,
        })
    }
}
