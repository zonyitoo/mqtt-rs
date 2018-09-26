extern crate mqtt;
#[macro_use]
extern crate log;
extern crate clap;
extern crate env_logger;
extern crate time;
extern crate uuid;

use std::env;
use std::io::Write;
use std::net::TcpStream;
use std::str;

use clap::{App, Arg};

use uuid::Uuid;

use mqtt::control::variable_header::ConnectReturnCode;
use mqtt::packet::*;
use mqtt::TopicFilter;
use mqtt::{Decodable, Encodable, QualityOfService};

use std::thread;
use std::time::Duration;

fn generate_client_id() -> String {
    format!("/MQTT/rust/{}", Uuid::new_v4())
}

fn main() {
    // configure logging
    env::set_var("RUST_LOG", env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()));
    env_logger::init();

    let matches = App::new("sub-client")
        .author("Y. T. Chung <zonyitoo@gmail.com>")
        .arg(
            Arg::with_name("SERVER")
                .short("S")
                .long("server")
                .takes_value(true)
                .required(true)
                .help("MQTT server address (host:port)"),
        ).arg(
            Arg::with_name("SUBSCRIBE")
                .short("s")
                .long("subscribe")
                .takes_value(true)
                .multiple(true)
                .required(true)
                .help("Channel filter to subscribe"),
        ).arg(
            Arg::with_name("USER_NAME")
                .short("u")
                .long("username")
                .takes_value(true)
                .help("Login user name"),
        ).arg(
            Arg::with_name("PASSWORD")
                .short("p")
                .long("password")
                .takes_value(true)
                .help("Password"),
        ).arg(
            Arg::with_name("CLIENT_ID")
                .short("i")
                .long("client-identifier")
                .takes_value(true)
                .help("Client identifier"),
        ).get_matches();

    let server_addr = matches.value_of("SERVER").unwrap();
    let client_id = matches
        .value_of("CLIENT_ID")
        .map(|x| x.to_owned())
        .unwrap_or_else(generate_client_id);
    let channel_filters: Vec<(TopicFilter, QualityOfService)> = matches
        .values_of("SUBSCRIBE")
        .unwrap()
        .map(|c| (TopicFilter::new(c.to_string()).unwrap(), QualityOfService::Level0))
        .collect();

    let keep_alive = 10;

    info!("Connecting to {:?} ... ", server_addr);
    let mut stream = TcpStream::connect(server_addr).unwrap();
    info!("Connected!");

    info!("Client identifier {:?}", client_id);
    let mut conn = ConnectPacket::new("MQTT", client_id);
    conn.set_clean_session(true);
    conn.set_keep_alive(keep_alive);
    let mut buf = Vec::new();
    conn.encode(&mut buf).unwrap();
    stream.write_all(&buf[..]).unwrap();

    let connack = ConnackPacket::decode(&mut stream).unwrap();
    trace!("CONNACK {:?}", connack);

    if connack.connect_return_code() != ConnectReturnCode::ConnectionAccepted {
        panic!(
            "Failed to connect to server, return code {:?}",
            connack.connect_return_code()
        );
    }

    // const CHANNEL_FILTER: &'static str = "typing-speed-test.aoeu.eu";
    info!("Applying channel filters {:?} ...", channel_filters);
    let sub = SubscribePacket::new(10, channel_filters);
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

        if let VariablePacket::SubackPacket(ref ack) = packet {
            if ack.packet_identifier() != 10 {
                panic!("SUBACK packet identifier not match");
            }

            info!("Subscribed!");
            break;
        }
    }

    let mut stream_clone = stream.try_clone().unwrap();
    thread::spawn(move || {
        let mut last_ping_time = 0;
        let mut next_ping_time = last_ping_time + (keep_alive as f32 * 0.9) as i64;
        loop {
            let current_timestamp = time::get_time().sec;
            if keep_alive > 0 && current_timestamp >= next_ping_time {
                info!("Sending PINGREQ to broker");

                let pingreq_packet = PingreqPacket::new();

                let mut buf = Vec::new();
                pingreq_packet.encode(&mut buf).unwrap();
                stream_clone.write_all(&buf[..]).unwrap();

                last_ping_time = current_timestamp;
                next_ping_time = last_ping_time + (keep_alive as f32 * 0.9) as i64;
                thread::sleep(Duration::new((keep_alive / 2) as u64, 0));
            }
        }
    });

    loop {
        let packet = match VariablePacket::decode(&mut stream) {
            Ok(pk) => pk,
            Err(err) => {
                error!("Error in receiving packet {}", err);
                continue;
            }
        };
        trace!("PACKET {:?}", packet);

        match packet {
            VariablePacket::PingrespPacket(..) => {
                info!("Receiving PINGRESP from broker ..");
            }
            VariablePacket::PublishPacket(ref publ) => {
                let msg = match str::from_utf8(&publ.payload_ref()[..]) {
                    Ok(msg) => msg,
                    Err(err) => {
                        error!("Failed to decode publish message {:?}", err);
                        continue;
                    }
                };
                info!("PUBLISH ({}): {}", publ.topic_name(), msg);
            }
            _ => {}
        }
    }
}
