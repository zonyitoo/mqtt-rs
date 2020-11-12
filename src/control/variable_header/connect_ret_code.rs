use std::convert::From;
use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::control::variable_header::VariableHeaderError;
use crate::{Decodable, Encodable};

pub const CONNECTION_ACCEPTED: u8 = 0x00;
pub const UNACCEPTABLE_PROTOCOL_VERSION: u8 = 0x01;
pub const IDENTIFIER_REJECTED: u8 = 0x02;
pub const SERVICE_UNAVAILABLE: u8 = 0x03;
pub const BAD_USER_NAME_OR_PASSWORD: u8 = 0x04;
pub const NOT_AUTHORIZED: u8 = 0x05;

/// Return code for `CONNACK` packet
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ConnectReturnCode {
    ConnectionAccepted,
    UnacceptableProtocolVersion,
    IdentifierRejected,
    ServiceUnavailable,
    BadUserNameOrPassword,
    NotAuthorized,
    Reserved(u8),
}

impl ConnectReturnCode {
    /// Get the code
    pub fn to_u8(self) -> u8 {
        match self {
            ConnectReturnCode::ConnectionAccepted => CONNECTION_ACCEPTED,
            ConnectReturnCode::UnacceptableProtocolVersion => UNACCEPTABLE_PROTOCOL_VERSION,
            ConnectReturnCode::IdentifierRejected => IDENTIFIER_REJECTED,
            ConnectReturnCode::ServiceUnavailable => SERVICE_UNAVAILABLE,
            ConnectReturnCode::BadUserNameOrPassword => BAD_USER_NAME_OR_PASSWORD,
            ConnectReturnCode::NotAuthorized => NOT_AUTHORIZED,
            ConnectReturnCode::Reserved(r) => r,
        }
    }

    /// Create `ConnectReturnCode` from code
    pub fn from_u8(code: u8) -> ConnectReturnCode {
        match code {
            CONNECTION_ACCEPTED => ConnectReturnCode::ConnectionAccepted,
            UNACCEPTABLE_PROTOCOL_VERSION => ConnectReturnCode::UnacceptableProtocolVersion,
            IDENTIFIER_REJECTED => ConnectReturnCode::IdentifierRejected,
            SERVICE_UNAVAILABLE => ConnectReturnCode::ServiceUnavailable,
            BAD_USER_NAME_OR_PASSWORD => ConnectReturnCode::BadUserNameOrPassword,
            NOT_AUTHORIZED => ConnectReturnCode::NotAuthorized,
            _ => ConnectReturnCode::Reserved(code),
        }
    }
}

impl Encodable for ConnectReturnCode {
    type Err = VariableHeaderError;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), VariableHeaderError> {
        writer.write_u8(self.to_u8()).map_err(From::from)
    }

    fn encoded_length(&self) -> u32 {
        1
    }
}

impl Decodable for ConnectReturnCode {
    type Err = VariableHeaderError;
    type Cond = ();

    fn decode_with<R: Read>(reader: &mut R, _rest: Option<()>) -> Result<ConnectReturnCode, VariableHeaderError> {
        reader.read_u8().map(ConnectReturnCode::from_u8).map_err(From::from)
    }
}
