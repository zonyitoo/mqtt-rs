extern crate mqtt;
#[macro_use]
extern crate log;
extern crate clap;
extern crate env_logger;
extern crate futures;
extern crate tokio;
extern crate uuid;

use std::env;
use std::fmt::Debug;
use std::io::Write;
use std::net;
use std::str;
use std::time::{Duration, Instant};

use clap::{App, Arg};

use uuid::Uuid;

use futures::{future, Future, Stream};

use tokio::io::{self, AsyncRead};
use tokio::net::TcpStream;
use tokio::timer::Interval;

use mqtt::control::variable_header::ConnectReturnCode;
use mqtt::packet::*;
use mqtt::TopicFilter;
use mqtt::{Decodable, Encodable, QualityOfService};

fn generate_client_id() -> String {
    format!("/MQTT/rust/{}", Uuid::new_v4())
}

fn alt_drop<E: Debug>(err: E) {
    warn!("{:?}", err);
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
    let mut stream = net::TcpStream::connect(server_addr).unwrap();
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

    // connection made, start the async work
    let program = future::ok(()).and_then(move |()| {
        let stream = TcpStream::from_std(stream, &Default::default()).unwrap();
        let (mqtt_read, mqtt_write) = stream.split();

        let ping_time = Duration::new((keep_alive / 2) as u64, 0);
        let ping_stream = Interval::new(Instant::now() + ping_time, ping_time);

        let ping_sender = ping_stream.map_err(alt_drop).fold(mqtt_write, |mqtt_write, _| {
            info!("Sending PINGREQ to broker");

            let pingreq_packet = PingreqPacket::new();

            let mut buf = Vec::new();
            pingreq_packet.encode(&mut buf).unwrap();
            io::write_all(mqtt_write, buf)
                .map(|(mqtt_write, _buf)| mqtt_write)
                .map_err(alt_drop)
        });

        let receiver = future::loop_fn::<_, (), _, _>(mqtt_read, |mqtt_read| {
            VariablePacket::parse(mqtt_read).map(|(mqtt_read, packet)| {
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
                                return future::Loop::Continue(mqtt_read);
                            }
                        };
                        info!("PUBLISH ({}): {}", publ.topic_name(), msg);
                    }
                    _ => {}
                }

                future::Loop::Continue(mqtt_read)
            })
        }).map_err(alt_drop);

        ping_sender.join(receiver).map(alt_drop)
    });

    tokio::run(program);
}
