# MQTT-rs

[![Build Status](https://img.shields.io/travis/zonyitoo/mqtt-rs.svg)](https://travis-ci.org/zonyitoo/mqtt-rs)
[![License](https://img.shields.io/github/license/zonyitoo/mqtt-rs.svg)](https://github.com/zonyitoo/mqtt-rs)

MQTT protocol library for Rust

```rust
[dependencies]
mqtt-protocol = "0.3"
```

## Usage

```rust
extern crate mqtt;

use std::io::Cursor;

use mqtt::{Encodable, Decodable};
use mqtt::packet::{VariablePacket, PublishPacket, QoSWithPacketIdentifier};
use mqtt::TopicName;

fn main() {
    // Create a new Publish packet
    let packet = PublishPacket::new(TopicName::new("mqtt/learning".to_owned()).unwrap(),
                                    QoSWithPacketIdentifier::Level2(10),
                                    b"Hello MQTT!".to_vec());

    // Encode
    let mut buf = Vec::new();
    packet.encode(&mut buf).unwrap();
    println!("Encoded: {:?}", buf);

    // Decode it with known type
    let mut dec_buf = Cursor::new(&buf[..]);
    let decoded = PublishPacket::decode(&mut dec_buf).unwrap();
    println!("Decoded: {:?}", decoded);
    assert_eq!(packet, decoded);

    // Auto decode by the fixed header
    let mut dec_buf = Cursor::new(&buf[..]);
    let auto_decode = VariablePacket::decode(&mut dec_buf).unwrap();
    println!("Variable packet decode: {:?}", auto_decode);
    assert_eq!(VariablePacket::PublishPacket(packet), auto_decode);
}
```

## Note

* Based on [MQTT 3.1.1](http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html)
