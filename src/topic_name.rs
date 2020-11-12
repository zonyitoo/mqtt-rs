//! Topic name

use std::convert::Into;
use std::error::Error;
use std::fmt;
use std::io::{Read, Write};
use std::ops::Deref;

use lazy_static::lazy_static;
use regex::Regex;

use crate::encodable::StringEncodeError;
use crate::{Decodable, Encodable};

const TOPIC_NAME_VALIDATE_REGEX: &str = r"^[^#+]+$";

lazy_static! {
    static ref TOPIC_NAME_VALIDATOR: Regex = Regex::new(TOPIC_NAME_VALIDATE_REGEX).unwrap();
}

#[inline]
fn is_invalid_topic_name(topic_name: &str) -> bool {
    topic_name.is_empty() || topic_name.as_bytes().len() > 65535 || !TOPIC_NAME_VALIDATOR.is_match(&topic_name)
}

/// Topic name
///
/// http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718106
#[derive(Debug, Eq, PartialEq, Clone, Hash, Ord, PartialOrd)]
pub struct TopicName(String);

impl TopicName {
    /// Creates a new topic name from string
    /// Return error if the string is not a valid topic name
    pub fn new<S: Into<String>>(topic_name: S) -> Result<TopicName, TopicNameError> {
        let topic_name = topic_name.into();
        if is_invalid_topic_name(&topic_name) {
            Err(TopicNameError::InvalidTopicName(topic_name))
        } else {
            Ok(TopicName(topic_name))
        }
    }

    /// Creates a new topic name from string without validation
    ///
    /// # Safety
    ///
    /// Topic names' syntax is defined in [MQTT specification](http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718106).
    /// Creating a name from raw string may cause errors
    pub unsafe fn new_unchecked(topic_name: String) -> TopicName {
        TopicName(topic_name)
    }
}

impl From<TopicName> for String {
    fn from(topic_name: TopicName) -> String {
        topic_name.0
    }
}

impl Deref for TopicName {
    type Target = TopicNameRef;

    fn deref(&self) -> &TopicNameRef {
        unsafe { TopicNameRef::new_unchecked(&self.0) }
    }
}

impl Encodable for TopicName {
    type Err = TopicNameError;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), TopicNameError> {
        (&self.0[..]).encode(writer).map_err(TopicNameError::StringEncodeError)
    }

    fn encoded_length(&self) -> u32 {
        (&self.0[..]).encoded_length()
    }
}

impl Decodable for TopicName {
    type Err = TopicNameError;
    type Cond = ();

    fn decode_with<R: Read>(reader: &mut R, _rest: Option<()>) -> Result<TopicName, TopicNameError> {
        let topic_name: String = Decodable::decode(reader).map_err(TopicNameError::StringEncodeError)?;
        TopicName::new(topic_name)
    }
}

/// Errors while parsing topic names
#[derive(Debug)]
pub enum TopicNameError {
    StringEncodeError(StringEncodeError),
    InvalidTopicName(String),
}

impl fmt::Display for TopicNameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TopicNameError::StringEncodeError(ref err) => err.fmt(f),
            TopicNameError::InvalidTopicName(ref topic) => write!(f, "Invalid topic filter ({})", topic),
        }
    }
}

impl Error for TopicNameError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            TopicNameError::StringEncodeError(ref err) => Some(err),
            TopicNameError::InvalidTopicName(..) => None,
        }
    }
}

/// Reference to a topic name
#[derive(Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct TopicNameRef(str);

impl TopicNameRef {
    /// Creates a new topic name from string
    /// Return error if the string is not a valid topic name
    pub fn new<S: AsRef<str> + ?Sized>(topic_name: &S) -> Result<&TopicNameRef, TopicNameError> {
        let topic_name = topic_name.as_ref();
        if is_invalid_topic_name(&topic_name) {
            Err(TopicNameError::InvalidTopicName(topic_name.to_owned()))
        } else {
            Ok(unsafe { &*(topic_name as *const str as *const TopicNameRef) })
        }
    }

    /// Creates a new topic name from string without validation
    ///
    /// # Safety
    ///
    /// Topic names' syntax is defined in [MQTT specification](http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718106).
    /// Creating a name from raw string may cause errors
    pub unsafe fn new_unchecked<S: AsRef<str> + ?Sized>(topic_name: &S) -> &TopicNameRef {
        let topic_name = topic_name.as_ref();
        &*(topic_name as *const str as *const TopicNameRef)
    }

    /// Check if this topic name is only for server.
    ///
    /// Topic names that beginning with a '$' character are reserved for servers
    pub fn is_server_specific(&self) -> bool {
        self.0.starts_with('$')
    }
}

impl Deref for TopicNameRef {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn topic_name_sys() {
        let topic_name = "$SYS".to_owned();
        TopicName::new(topic_name).unwrap();

        let topic_name = "$SYS/broker/connection/test.cosm-energy/state".to_owned();
        TopicName::new(topic_name).unwrap();
    }

    #[test]
    fn topic_name_slash() {
        TopicName::new("/").unwrap();
    }

    #[test]
    fn topic_name_basic() {
        TopicName::new("/finance").unwrap();
        TopicName::new("/finance//def").unwrap();
    }
}
