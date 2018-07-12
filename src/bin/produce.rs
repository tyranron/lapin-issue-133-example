extern crate amq_protocol;
extern crate futures;
extern crate lapin_futures as lapin;
#[macro_use]
extern crate log;
extern crate stderrlog;
extern crate tokio;

use std::str::FromStr;
use std::time;

use amq_protocol::uri::AMQPUri;
use futures::future::{self, Future, Loop};
use tokio::{net::TcpStream, timer::Delay};
use lapin::{
    client::{Client, ConnectionOptions},
    channel::{BasicPublishOptions, BasicProperties},
};

const DEFAULT_ADDR: &'static str =
    "amqp://admin:qweqweqwe@127.0.0.1//?heartbeat=5";

const DEFAULT_MSG: &str = "example message";

fn main() {
    stderrlog::new().verbosity(2).init().unwrap();

    let uri = AMQPUri::from_str(DEFAULT_ADDR).unwrap();
    let addr = format!("{}:{}", uri.authority.host, uri.authority.port)
        .parse().unwrap();
    let opts = ConnectionOptions::from_uri(uri);

    tokio::run(TcpStream::connect(&addr)
        .and_then(|stream| Client::connect(stream, opts))
        .and_then(|(client, heartbeat)| {
            tokio::spawn(heartbeat.map_err(|e| {
                error!("Heartbeat failed: {}", e)
            }));
            client.create_channel()
        })
        .map_err(|e| error!("Connection failed: {}", e))
        .and_then(|ch| {
            future::loop_fn((), move |_| {
                ch.basic_publish(
                    "example-exchange", "route_to_everybody",
                    DEFAULT_MSG.as_bytes().to_vec(),
                    BasicPublishOptions::default(),
                    BasicProperties::default()
                ).map_err(|e| {
                    error!("Send failed: {}", e)
                }).and_then(|_| {
                    info!("Sent msg");
                    Delay::new(time::Instant::now()
                                   + time::Duration::from_millis(500))
                        .map_err(|e| error!("Delay failed: {}", e))
                }).and_then(|_| Ok(Loop::Continue(())))
            })
        })
        .map(|_: ()| info!("Done!")))
}
