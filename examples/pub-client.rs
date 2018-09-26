extern crate mqtt;
#[macro_use]
extern crate log;
extern crate clap;
extern crate env_logger;
extern crate uuid;

use std::env;
use std::io::{self, Write};
use std::net::TcpStream;
use std::thread;

use clap::{App, Arg};

use uuid::Uuid;

use mqtt::control::variable_header::ConnectReturnCode;
use mqtt::packet::*;
use mqtt::{Decodable, Encodable, QualityOfService};
use mqtt::{TopicFilter, TopicName};

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

    info!("Connecting to {:?} ... ", server_addr);
    let mut stream = TcpStream::connect(server_addr).unwrap();
    info!("Connected!");

    info!("Client identifier {:?}", client_id);
    let mut conn = ConnectPacket::new("MQTT", client_id);
    conn.set_clean_session(true);
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

    info!("Applying channel filters {:?} ...", channel_filters);
    let sub = SubscribePacket::new(10, channel_filters);
    let mut buf = Vec::new();
    sub.encode(&mut buf).unwrap();
    stream.write_all(&buf[..]).unwrap();

    let channels: Vec<TopicName> = matches
        .values_of("SUBSCRIBE")
        .unwrap()
        .map(|c| TopicName::new(c.to_string()).unwrap())
        .collect();

    let user_name = matches.value_of("USER_NAME").unwrap_or("<anonym>");

    let mut cloned_stream = stream.try_clone().unwrap();
    thread::spawn(move || {
        loop {
            let packet = match VariablePacket::decode(&mut cloned_stream) {
                Ok(pk) => pk,
                Err(err) => {
                    error!("Error in receiving packet {:?}", err);
                    continue;
                }
            };
            trace!("PACKET {:?}", packet);

            match packet {
                VariablePacket::PingreqPacket(..) => {
                    let pingresp = PingrespPacket::new();
                    info!("Sending Ping response {:?}", pingresp);
                    pingresp.encode(&mut cloned_stream).unwrap();
                }
                VariablePacket::DisconnectPacket(..) => {
                    break;
                }
                _ => {
                    // Ignore other packets in pub client
                }
            }
        }
    });

    let stdin = io::stdin();
    loop {
        print!("{}: ", user_name);
        io::stdout().flush().unwrap();

        let mut line = String::new();
        stdin.read_line(&mut line).unwrap();

        if line.trim_right() == "" {
            continue;
        }

        let message = format!("{}: {}", user_name, line.trim_right());

        for chan in &channels {
            let publish_packet = PublishPacket::new(chan.clone(), QoSWithPacketIdentifier::Level0, message.clone());
            let mut buf = Vec::new();
            publish_packet.encode(&mut buf).unwrap();
            stream.write_all(&buf[..]).unwrap();
        }
    }
}
