//! Topic filter

use std::convert::Into;
use std::error::Error;
use std::fmt;
use std::io::{Read, Write};
use std::ops::Deref;

use lazy_static::lazy_static;
use regex::Regex;

use crate::encodable::StringEncodeError;
use crate::topic_name::TopicNameRef;
use crate::{Decodable, Encodable};

const VALIDATE_TOPIC_FILTER_REGEX: &str = r"^(([^+#]*|\+)(/([^+#]*|\+))*(/#)?|#)$";

lazy_static! {
    static ref TOPIC_FILTER_VALIDATOR: Regex = Regex::new(VALIDATE_TOPIC_FILTER_REGEX).unwrap();
}

#[inline]
fn is_invalid_topic_filter(topic: &str) -> bool {
    topic.is_empty() || topic.as_bytes().len() > 65535 || !TOPIC_FILTER_VALIDATOR.is_match(&topic)
}

/// Topic filter
///
/// http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718106
///
/// ```rust
/// use mqtt::{TopicFilter, TopicNameRef};
///
/// let topic_filter = TopicFilter::new("sport/+/player1").unwrap();
/// let matcher = topic_filter.get_matcher();
/// assert!(matcher.is_match(TopicNameRef::new("sport/abc/player1").unwrap()));
/// ```
#[derive(Debug, Eq, PartialEq, Clone, Hash, Ord, PartialOrd)]
pub struct TopicFilter(String);

impl TopicFilter {
    /// Creates a new topic filter from string
    /// Return error if it is not a valid topic filter
    pub fn new<S: Into<String>>(topic: S) -> Result<TopicFilter, TopicFilterError> {
        let topic = topic.into();
        if is_invalid_topic_filter(&topic) {
            Err(TopicFilterError::InvalidTopicFilter(topic))
        } else {
            Ok(TopicFilter(topic))
        }
    }

    /// Creates a new topic filter from string without validation
    ///
    /// # Safety
    ///
    /// Topic filters' syntax is defined in [MQTT specification](http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718106).
    /// Creating a filter from raw string may cause errors
    pub unsafe fn new_unchecked<S: Into<String>>(topic: S) -> TopicFilter {
        TopicFilter(topic.into())
    }
}

impl From<TopicFilter> for String {
    fn from(topic: TopicFilter) -> String {
        topic.0
    }
}

impl Encodable for TopicFilter {
    type Err = TopicFilterError;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), TopicFilterError> {
        (&self.0[..])
            .encode(writer)
            .map_err(TopicFilterError::StringEncodeError)
    }

    fn encoded_length(&self) -> u32 {
        (&self.0[..]).encoded_length()
    }
}

impl Decodable for TopicFilter {
    type Err = TopicFilterError;
    type Cond = ();

    fn decode_with<R: Read>(reader: &mut R, _rest: Option<()>) -> Result<TopicFilter, TopicFilterError> {
        let topic_filter: String = Decodable::decode(reader).map_err(TopicFilterError::StringEncodeError)?;
        TopicFilter::new(topic_filter)
    }
}

impl Deref for TopicFilter {
    type Target = TopicFilterRef;

    fn deref(&self) -> &TopicFilterRef {
        unsafe { TopicFilterRef::new_unchecked(&self.0) }
    }
}

/// Reference to a `TopicFilter`
#[derive(Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct TopicFilterRef(str);

impl TopicFilterRef {
    /// Creates a new topic filter from string
    /// Return error if it is not a valid topic filter
    pub fn new<S: AsRef<str> + ?Sized>(topic: &S) -> Result<&TopicFilterRef, TopicFilterError> {
        let topic = topic.as_ref();
        if is_invalid_topic_filter(topic) {
            Err(TopicFilterError::InvalidTopicFilter(topic.to_owned()))
        } else {
            Ok(unsafe { &*(topic as *const str as *const TopicFilterRef) })
        }
    }

    /// Creates a new topic filter from string without validation
    ///
    /// # Safety
    ///
    /// Topic filters' syntax is defined in [MQTT specification](http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718106).
    /// Creating a filter from raw string may cause errors
    pub unsafe fn new_unchecked<S: AsRef<str> + ?Sized>(topic: &S) -> &TopicFilterRef {
        let topic = topic.as_ref();
        &*(topic as *const str as *const TopicFilterRef)
    }

    /// Get a matcher
    pub fn get_matcher(&self) -> TopicFilterMatcher<'_> {
        TopicFilterMatcher::new(&self.0)
    }
}

impl Deref for TopicFilterRef {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

/// Errors while parsing topic filters
#[derive(Debug)]
pub enum TopicFilterError {
    StringEncodeError(StringEncodeError),
    InvalidTopicFilter(String),
}

impl fmt::Display for TopicFilterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TopicFilterError::StringEncodeError(ref err) => err.fmt(f),
            TopicFilterError::InvalidTopicFilter(ref topic) => write!(f, "Invalid topic filter ({})", topic),
        }
    }
}

impl Error for TopicFilterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            TopicFilterError::StringEncodeError(ref err) => Some(err),
            TopicFilterError::InvalidTopicFilter(..) => None,
        }
    }
}

/// Matcher for matching topic names with this filter
#[derive(Debug, Copy, Clone)]
pub struct TopicFilterMatcher<'a> {
    topic_filter: &'a str,
}

impl<'a> TopicFilterMatcher<'a> {
    fn new(filter: &'a str) -> TopicFilterMatcher<'a> {
        TopicFilterMatcher { topic_filter: filter }
    }

    /// Check if this filter can match the `topic_name`
    pub fn is_match(&self, topic_name: &TopicNameRef) -> bool {
        let mut tn_itr = topic_name.split('/');
        let mut ft_itr = self.topic_filter.split('/');

        // The Server MUST NOT match Topic Filters starting with a wildcard character (# or +)
        // with Topic Names beginning with a $ character [MQTT-4.7.2-1].

        let first_ft = ft_itr.next().unwrap();
        let first_tn = tn_itr.next().unwrap();

        if first_tn.starts_with('$') {
            if first_tn != first_ft {
                return false;
            }
        } else {
            match first_ft {
                // Matches the whole topic
                "#" => return true,
                "+" => {}
                _ => {
                    if first_tn != first_ft {
                        return false;
                    }
                }
            }
        }

        loop {
            match (ft_itr.next(), tn_itr.next()) {
                (Some(ft), Some(tn)) => match ft {
                    "#" => break,
                    "+" => {}
                    _ => {
                        if ft != tn {
                            return false;
                        }
                    }
                },
                (Some(ft), None) => {
                    if ft != "#" {
                        return false;
                    } else {
                        break;
                    }
                }
                (None, Some(..)) => return false,
                (None, None) => break,
            }
        }

        true
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn topic_filter_validate() {
        let topic = "#".to_owned();
        TopicFilter::new(topic).unwrap();

        let topic = "sport/tennis/player1".to_owned();
        TopicFilter::new(topic).unwrap();

        let topic = "sport/tennis/player1/ranking".to_owned();
        TopicFilter::new(topic).unwrap();

        let topic = "sport/tennis/player1/#".to_owned();
        TopicFilter::new(topic).unwrap();

        let topic = "#".to_owned();
        TopicFilter::new(topic).unwrap();

        let topic = "sport/tennis/#".to_owned();
        TopicFilter::new(topic).unwrap();

        let topic = "sport/tennis#".to_owned();
        assert!(TopicFilter::new(topic).is_err());

        let topic = "sport/tennis/#/ranking".to_owned();
        assert!(TopicFilter::new(topic).is_err());

        let topic = "+".to_owned();
        TopicFilter::new(topic).unwrap();

        let topic = "+/tennis/#".to_owned();
        TopicFilter::new(topic).unwrap();

        let topic = "sport+".to_owned();
        assert!(TopicFilter::new(topic).is_err());

        let topic = "sport/+/player1".to_owned();
        TopicFilter::new(topic).unwrap();

        let topic = "+/+".to_owned();
        TopicFilter::new(topic).unwrap();

        let topic = "$SYS/#".to_owned();
        TopicFilter::new(topic).unwrap();

        let topic = "$SYS".to_owned();
        TopicFilter::new(topic).unwrap();
    }

    #[test]
    fn topic_filter_matcher() {
        let filter = TopicFilter::new("sport/#").unwrap();
        let matcher = filter.get_matcher();
        assert!(matcher.is_match(TopicNameRef::new("sport").unwrap()));

        let filter = TopicFilter::new("#").unwrap();
        let matcher = filter.get_matcher();
        assert!(matcher.is_match(TopicNameRef::new("sport").unwrap()));
        assert!(matcher.is_match(TopicNameRef::new("/").unwrap()));
        assert!(matcher.is_match(TopicNameRef::new("abc/def").unwrap()));
        assert!(!matcher.is_match(TopicNameRef::new("$SYS").unwrap()));
        assert!(!matcher.is_match(TopicNameRef::new("$SYS/abc").unwrap()));

        let filter = TopicFilter::new("+/monitor/Clients").unwrap();
        let matcher = filter.get_matcher();
        assert!(!matcher.is_match(TopicNameRef::new("$SYS/monitor/Clients").unwrap()));

        let filter = TopicFilter::new("$SYS/#").unwrap();
        let matcher = filter.get_matcher();
        assert!(matcher.is_match(TopicNameRef::new("$SYS/monitor/Clients").unwrap()));
        assert!(matcher.is_match(TopicNameRef::new("$SYS").unwrap()));

        let filter = TopicFilter::new("$SYS/monitor/+").unwrap();
        let matcher = filter.get_matcher();
        assert!(matcher.is_match(TopicNameRef::new("$SYS/monitor/Clients").unwrap()));
    }
}
