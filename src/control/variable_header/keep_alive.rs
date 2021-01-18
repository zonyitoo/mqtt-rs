use std::io::{self, Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::control::variable_header::VariableHeaderError;
use crate::{Decodable, Encodable};

/// Keep alive time interval
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct KeepAlive(pub u16);

impl Encodable for KeepAlive {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_u16::<BigEndian>(self.0)
    }

    fn encoded_length(&self) -> u32 {
        2
    }
}

impl Decodable for KeepAlive {
    type Error = VariableHeaderError;
    type Cond = ();

    fn decode_with<R: Read>(reader: &mut R, _rest: ()) -> Result<KeepAlive, VariableHeaderError> {
        reader.read_u16::<BigEndian>().map(KeepAlive).map_err(From::from)
    }
}
