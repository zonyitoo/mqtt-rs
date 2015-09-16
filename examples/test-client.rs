extern crate mqtt;
#[macro_use]
extern crate log;
extern crate env_logger;

use std::net::TcpStream;
use std::io::Write;
use std::str;

use mqtt::{Encodable, Decodable, QualityOfService};
use mqtt::packet::*;
use mqtt::control::variable_header::ConnectReturnCode;

fn main() {
    env_logger::init().unwrap();

    const SERVER_ADDR: &'static str = "test.mosquitto.org:1883";

    print!("Connecting to {:?} ... ", SERVER_ADDR);
    let mut stream = TcpStream::connect(SERVER_ADDR).unwrap();
    println!("Connected!");

    const CLIENT_ID: &'static str = "zonyitoo_0001";
    println!("Client identifier {:?}", CLIENT_ID);
    let mut conn = ConnectPacket::new(CLIENT_ID.to_owned());
    conn.set_clean_session(true);
    let mut buf = Vec::new();
    conn.encode(&mut buf).unwrap();
    stream.write_all(&buf[..]).unwrap();

    let connack = ConnackPacket::decode(&mut stream).unwrap();
    trace!("CONNACK {:?}", connack);

    if connack.connect_return_code() != ConnectReturnCode::ConnectionAccepted {
        panic!("Failed to connect to server, return code {:?}", connack.connect_return_code());
    }

    const CHANNEL_FILTER: &'static str = "typing-speed-test.aoeu.eu";
    println!("Subscribing {:?} channel ...", CHANNEL_FILTER);
    let sub = SubscribePacket::new(10, vec![(CHANNEL_FILTER.to_owned(), QualityOfService::Level2)]);
    let mut buf = Vec::new();
    sub.encode(&mut buf).unwrap();
    stream.write_all(&buf[..]).unwrap();

    loop {
        let packet = match VariablePacket::decode(&mut stream) {
            Ok(pk) => pk,
            Err(err) => {
            error!("Error in receiving packet {:?}", err);
                continue;
            }
        };
        trace!("PACKET {:?}", packet);

        match &packet {
            &VariablePacket::SubackPacket(ref ack) => {
                if ack.packet_identifier() != 10 {
                    panic!("SUBACK packet identifier not match");
                }

                println!("Subscribed!");
                break;
            },
            _ => {}
        }
    }

    loop {
        let packet = match VariablePacket::decode(&mut stream) {
            Ok(pk) => pk,
            Err(err) => {
            error!("Error in receiving packet {:?}", err);
                continue;
            }
        };
        trace!("PACKET {:?}", packet);

        match &packet {
            &VariablePacket::PingreqPacket(..) => {
                let pingresp = PingrespPacket::new();
                info!("Sending Ping response {:?}", pingresp);
                pingresp.encode(&mut stream).unwrap();
            },
            &VariablePacket::PublishPacket(ref publ) => {
                let msg = match str::from_utf8(&publ.payload()[..]) {
                    Ok(msg) => msg,
                    Err(err) => {
                        error!("Failed to decode publish message {:?}", err);
                        continue;
                    }
                };
                println!("PUBLISH ({}): {}", publ.topic_name(), msg);
            },
            _ => {}
        }
    }
}
