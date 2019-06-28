//! Variable header in MQTT

use std::convert::From;
use std::error::Error;
use std::fmt;
use std::io;

use encodable::StringCodecError;
use topic_name::TopicNameError;

pub use self::connect_ack_flags::ConnackFlags;
pub use self::connect_flags::ConnectFlags;
pub use self::connect_ret_code::ConnectReturnCode;
pub use self::keep_alive::KeepAlive;
pub use self::packet_identifier::PacketIdentifier;
pub use self::protocol_level::ProtocolLevel;
pub use self::protocol_name::ProtocolName;
pub use self::topic_name::TopicNameHeader;

mod connect_ack_flags;
mod connect_flags;
mod connect_ret_code;
mod keep_alive;
mod packet_identifier;
pub mod protocol_level;
mod protocol_name;
mod topic_name;

/// Errors while decoding variable header
#[derive(Debug)]
pub enum VariableHeaderError {
    IoError(io::Error),
    StringCodecError(StringCodecError),
    InvalidReservedFlag,
    TopicNameError(TopicNameError),
}

impl From<io::Error> for VariableHeaderError {
    fn from(err: io::Error) -> VariableHeaderError {
        VariableHeaderError::IoError(err)
    }
}

impl From<StringCodecError> for VariableHeaderError {
    fn from(err: StringCodecError) -> VariableHeaderError {
        VariableHeaderError::StringCodecError(err)
    }
}

impl From<TopicNameError> for VariableHeaderError {
    fn from(err: TopicNameError) -> VariableHeaderError {
        VariableHeaderError::TopicNameError(err)
    }
}

impl fmt::Display for VariableHeaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &VariableHeaderError::IoError(ref err) => write!(f, "{}", err),
            &VariableHeaderError::StringCodecError(ref err) => write!(f, "{}", err),
            &VariableHeaderError::InvalidReservedFlag => write!(f, "Invalid reserved flags"),
            &VariableHeaderError::TopicNameError(ref err) => write!(f, "{}", err),
        }
    }
}

impl Error for VariableHeaderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            &VariableHeaderError::IoError(ref err) => Some(err),
            &VariableHeaderError::StringCodecError(ref err) => Some(err),
            &VariableHeaderError::InvalidReservedFlag => None,
            &VariableHeaderError::TopicNameError(ref err) => Some(err),
        }
    }
}
