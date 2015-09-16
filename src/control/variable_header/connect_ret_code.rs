use std::io::{Read, Write};
use std::convert::From;

use byteorder::{ReadBytesExt, WriteBytesExt};

use control::variable_header::VariableHeaderError;
use {Encodable, Decodable};

pub const CONNECTION_ACCEPTED: u8 = 0x00;
pub const UNACCEPTABLE_PROTOCOL_VERSION: u8 = 0x01;
pub const IDENTIFIER_REJECTED: u8 = 0x02;
pub const SERVICE_UNAVAILABLE: u8 = 0x03;
pub const BAD_USER_NAME_OR_PASSWORD: u8 = 0x04;
pub const NOT_AUTHORIZED: u8 = 0x05;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct ConnectReturnCode(pub u8);

impl<'a> Encodable<'a> for ConnectReturnCode {
    type Err = VariableHeaderError;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), VariableHeaderError> {
        writer.write_u8(self.0)
            .map_err(From::from)
    }

    fn encoded_length(&self) -> u32 {
        1
    }
}

impl<'a> Decodable<'a> for ConnectReturnCode {
    type Err = VariableHeaderError;
    type Cond = ();

    fn decode_with<R: Read>(reader: &mut R, _rest: Option<()>) -> Result<ConnectReturnCode, VariableHeaderError> {
        reader.read_u8()
            .map(ConnectReturnCode)
            .map_err(From::from)
    }
}
