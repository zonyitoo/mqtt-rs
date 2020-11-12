//! MQTT protocol utilities library
//!
//! Strictly implements protocol of [MQTT v3.1.1](http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html)
//!
//! ## Usage
//!
//! ```rust
//! use std::io::Cursor;
//!
//! use mqtt::{Encodable, Decodable};
//! use mqtt::packet::{VariablePacket, PublishPacket, QoSWithPacketIdentifier};
//! use mqtt::TopicName;
//!
//! // Create a new Publish packet
//! let packet = PublishPacket::new(TopicName::new("mqtt/learning").unwrap(),
//!                                 QoSWithPacketIdentifier::Level2(10),
//!                                 b"Hello MQTT!".to_vec());
//!
//! // Encode
//! let mut buf = Vec::new();
//! packet.encode(&mut buf).unwrap();
//! println!("Encoded: {:?}", buf);
//!
//! // Decode it with known type
//! let mut dec_buf = Cursor::new(&buf[..]);
//! let decoded = PublishPacket::decode(&mut dec_buf).unwrap();
//! println!("Decoded: {:?}", decoded);
//! assert_eq!(packet, decoded);
//!
//! // Auto decode by the fixed header
//! let mut dec_buf = Cursor::new(&buf[..]);
//! let auto_decode = VariablePacket::decode(&mut dec_buf).unwrap();
//! println!("Variable packet decode: {:?}", auto_decode);
//! assert_eq!(VariablePacket::PublishPacket(packet), auto_decode);
//! ```

pub use self::encodable::{Decodable, Encodable};
pub use self::qos::QualityOfService;
pub use self::topic_filter::{TopicFilter, TopicFilterRef};
pub use self::topic_name::{TopicName, TopicNameRef};

pub mod control;
pub mod encodable;
pub mod packet;
pub mod qos;
pub mod topic_filter;
pub mod topic_name;
