use std::io::{Read, Write};
use std::convert::From;
use std::ops::Deref;

use regex::Regex;

use control::variable_header::VariableHeaderError;
use {Encodable, Decodable};

const TOPIC_NAME_VALIDATE_REGEX: &'static str = r"^(\$?[:alnum:]+)?(/[:alnum:]+)*$";

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct TopicName(String);

impl TopicName {
    pub fn new(topic_name: String) -> Result<TopicName, VariableHeaderError> {
        let re = Regex::new(TOPIC_NAME_VALIDATE_REGEX).unwrap();
        if topic_name.is_empty() || topic_name.as_bytes().len() > 65535 || !re.is_match(&topic_name[..]) {
            Err(VariableHeaderError::InvalidTopicName)
        } else {
            Ok(TopicName(topic_name))
        }
    }

    pub unsafe fn new_unchecked(topic_name: String) -> TopicName {
        TopicName(topic_name)
    }

    pub fn is_server_specific(&self) -> bool {
        self.0.starts_with('$')
    }
}

impl Deref for TopicName {
    type Target = String;

    fn deref(&self) -> &String {
        &self.0
    }
}

pub struct TopicNameRef<'a>(&'a str);

impl<'a> TopicNameRef<'a> {
    pub fn is_server_specific(&self) -> bool {
        self.0.starts_with('$')
    }
}

impl<'a> Encodable<'a> for TopicName {
    type Err = VariableHeaderError;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), VariableHeaderError> {
        (&self.0[..]).encode(writer).map_err(From::from)
    }

    fn encoded_length(&self) -> u32 {
        (&self.0[..]).encoded_length()
    }
}

impl<'a> Decodable<'a> for TopicName {
    type Err = VariableHeaderError;
    type Cond = ();

    fn decode_with<R: Read>(reader: &mut R, _rest: Option<()>) -> Result<TopicName, VariableHeaderError> {
        TopicName::new(try!(Decodable::decode(reader)))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_topic_name_basic() {
        let topic_name = "$SYS".to_owned();
        TopicName::new(topic_name).unwrap();
    }
}
