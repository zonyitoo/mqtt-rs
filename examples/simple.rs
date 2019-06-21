extern crate mqtt;

use std::io::Cursor;

use mqtt::packet::{PublishPacket, QoSWithPacketIdentifier, VariablePacket};
use mqtt::TopicName;
use mqtt::{Decodable, Encodable};

fn main() {
    // Create a new Publish packet
    let packet = PublishPacket::new(
        TopicName::new("mqtt/learning").unwrap(),
        QoSWithPacketIdentifier::Level2(10),
        "Hello MQTT!",
    );

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
