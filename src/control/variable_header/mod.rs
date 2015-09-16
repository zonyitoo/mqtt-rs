use std::convert::From;
use std::error::Error;
use std::io::{self, Write};
use std::fmt;
use std::string::FromUtf8Error;

use byteorder;

use Encodable;
use encodable::StringEncodeError;

pub use self::packet_identifier::PacketIdentifier;
pub use self::protocol_name::ProtocolName;
pub use self::protocol_level::ProtocolLevel;
pub use self::connect_flags::ConnectFlags;
pub use self::keep_alive::KeepAlive;
pub use self::connect_ack_flags::ConnackFlags;
pub use self::connect_ret_code::ConnectReturnCode;

pub mod packet_identifier;
pub mod protocol_name;
pub mod protocol_level;
pub mod connect_flags;
pub mod keep_alive;
pub mod connect_ack_flags;
pub mod connect_ret_code;

macro_rules! impl_variable_headers {
    ($($name:ident => $repr:ty,)*) => {
        /// Some types of MQTT Control Packets contain a variable header component.
        /// It resides between the fixed header and the payload. The content of the
        /// variable header varies depending on the Packet type. The Packet Identifier
        /// field of variable header is common in several packet types.
        #[derive(Debug, Eq, PartialEq, Clone)]
        pub enum VariableHeader {
            $(
                $name($repr),
            )*
        }

        impl VariableHeader {
            /// Create a VariableHeader
            pub fn new<H>(vhead: H) -> VariableHeader
                where VariableHeader: From<H>
            {
                From::from(vhead)
            }
        }

        $(
            impl From<$repr> for VariableHeader {
                fn from(vhead: $repr) -> VariableHeader {
                    VariableHeader::$name(vhead)
                }
            }
        )*

        impl<'a> Encodable<'a> for VariableHeader {
            type Err = VariableHeaderError;

            fn encode<W: Write>(&self, writer: &mut W) -> Result<(), VariableHeaderError> {
                match self {
                    $(
                        &VariableHeader::$name(ref repr) => repr.encode(writer),
                    )*
                }
            }

            fn encoded_length(&self) -> u32 {
                match self {
                    $(
                        &VariableHeader::$name(ref repr) => repr.encoded_length(),
                    )*
                }
            }
        }
    }
}

impl_variable_headers! {
    PacketIdentifier    => PacketIdentifier,
    ProtocolName        => ProtocolName,
    ProtocolLevel       => ProtocolLevel,
    ConnectFlags        => ConnectFlags,
    KeepAlive           => KeepAlive,
    ConnackFlags        => ConnackFlags,
    ConnectReturnCode   => ConnectReturnCode,
}

#[derive(Debug)]
pub enum VariableHeaderError {
    IoError(io::Error),
    StringEncodeError(StringEncodeError),
    InvalidReservedFlag,
    FromUtf8Error(FromUtf8Error),
}

impl From<io::Error> for VariableHeaderError {
    fn from(err: io::Error) -> VariableHeaderError {
        VariableHeaderError::IoError(err)
    }
}

impl From<byteorder::Error> for VariableHeaderError {
    fn from(err: byteorder::Error) -> VariableHeaderError {
        VariableHeaderError::IoError(From::from(err))
    }
}

impl From<FromUtf8Error> for VariableHeaderError {
    fn from(err: FromUtf8Error) -> VariableHeaderError {
        VariableHeaderError::FromUtf8Error(err)
    }
}

impl From<StringEncodeError> for VariableHeaderError {
    fn from(err: StringEncodeError) -> VariableHeaderError {
        VariableHeaderError::StringEncodeError(err)
    }
}

impl fmt::Display for VariableHeaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &VariableHeaderError::IoError(ref err) => write!(f, "{}", err),
            &VariableHeaderError::StringEncodeError(ref err) => write!(f, "{}", err),
            &VariableHeaderError::InvalidReservedFlag => write!(f, "Invalid reserved flags"),
            &VariableHeaderError::FromUtf8Error(ref err) => write!(f, "{}", err),
        }
    }
}

impl Error for VariableHeaderError {
    fn description(&self) -> &str {
        match self {
            &VariableHeaderError::IoError(ref err) => err.description(),
            &VariableHeaderError::StringEncodeError(ref err) => err.description(),
            &VariableHeaderError::InvalidReservedFlag => "Invalid reserved flags",
            &VariableHeaderError::FromUtf8Error(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match self {
            &VariableHeaderError::IoError(ref err) => Some(err),
            &VariableHeaderError::StringEncodeError(ref err) => Some(err),
            &VariableHeaderError::InvalidReservedFlag => None,
            &VariableHeaderError::FromUtf8Error(ref err) => Some(err),
        }
    }
}
