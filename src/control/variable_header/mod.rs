//! Variable header in MQTT

use std::io;
use std::string::FromUtf8Error;

use crate::topic_name::{TopicNameDecodeError, TopicNameError};

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
#[derive(Debug, thiserror::Error)]
pub enum VariableHeaderError {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error("invalid reserved flags")]
    InvalidReservedFlag,
    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),
    #[error(transparent)]
    TopicNameError(#[from] TopicNameError),
    #[error("invalid protocol version")]
    InvalidProtocolVersion,
}

impl From<TopicNameDecodeError> for VariableHeaderError {
    fn from(err: TopicNameDecodeError) -> VariableHeaderError {
        match err {
            TopicNameDecodeError::IoError(e) => Self::IoError(e),
            TopicNameDecodeError::InvalidTopicName(e) => Self::TopicNameError(e),
        }
    }
}
