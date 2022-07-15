use std::io::{self, Read, Write};

use crate::control::variable_header::VariableHeaderError;
use crate::topic_name::TopicName;
use crate::{Decodable, Encodable};

/// Topic name wrapper
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct TopicNameHeader(TopicName);

impl TopicNameHeader {
    pub fn new(topic_name: String) -> Result<TopicNameHeader, VariableHeaderError> {
        match TopicName::new(topic_name) {
            Ok(h) => Ok(TopicNameHeader(h)),
            Err(err) => Err(VariableHeaderError::TopicNameError(err)),
        }
    }
}

impl From<TopicNameHeader> for TopicName {
    fn from(hdr: TopicNameHeader) -> Self {
        hdr.0
    }
}

impl Encodable for TopicNameHeader {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        (&self.0[..]).encode(writer)
    }

    fn encoded_length(&self) -> u32 {
        (&self.0[..]).encoded_length()
    }
}

impl Decodable for TopicNameHeader {
    type Error = VariableHeaderError;
    type Cond = ();

    fn decode_with<R: Read>(reader: &mut R, _rest: ()) -> Result<TopicNameHeader, VariableHeaderError> {
        TopicNameHeader::new(Decodable::decode(reader)?)
    }
}
