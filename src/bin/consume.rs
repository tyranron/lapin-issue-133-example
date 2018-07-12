extern crate amq_protocol;
extern crate crossbeam_channel as crossbeam;
extern crate futures;
extern crate lapin_async;
extern crate lapin_futures as lapin;
#[macro_use]
extern crate log;
extern crate stderrlog;
extern crate tokio;

use std::str::FromStr;
use std::thread;

use amq_protocol::{
    types::{AMQPValue, FieldTable},
    uri::AMQPUri,
};
use futures::{future::Future, stream::Stream};
use lapin::{
    channel,
    client::{Client, ConnectionOptions},
};
use tokio::{executor::current_thread, net::TcpStream};

const DEFAULT_ADDR: &str = "amqp://consumer:qweqwe@127.0.0.1//?heartbeat=5";

fn main() {
    stderrlog::new().verbosity(2).init().unwrap();

    let (send_to, receive_from) = crossbeam::bounded(0);
    for _ in 0..2 {
        let receive_from = receive_from.clone();
        thread::spawn(move || process_sync(receive_from));
    }

    let uri = AMQPUri::from_str(DEFAULT_ADDR).unwrap();
    let addr = format!("{}:{}", uri.authority.host, uri.authority.port)
        .parse()
        .unwrap();
    let opts = ConnectionOptions::from_uri(uri);

    tokio::run(
        TcpStream::connect(&addr)
            .and_then(|stream| Client::connect(stream, opts))
            .and_then(|(client, heartbeat)| {
                tokio::spawn(
                    heartbeat.map_err(|e| error!("Heartbeat failed: {}", e)),
                );
                client.create_channel()
            })
            .map_err(|e| error!("Connection failed: {}", e))
            .and_then(move |ch| {
                let opts = channel::QueueDeclareOptions {
                    passive: true,
                    durable: true,
                    auto_delete: false,
                    ..channel::QueueDeclareOptions::default()
                };
                let mut args = FieldTable::new();
                args.insert(
                    "x-message-ttl".into(),
                    AMQPValue::LongUInt(3_600_000),
                );

                let ack_ch = ch.clone();
                ch.queue_declare("example-queue", opts, args)
                    .and_then(move |queue| {
                        ch.basic_consume(
                            &queue,
                            "",
                            channel::BasicConsumeOptions::default(),
                            FieldTable::new(),
                        )
                    })
                    .and_then(move |stream| {
                        stream.for_each(move |msg| {
                            info!("Consumed msg '{}'", msg.delivery_tag);
                            send_to.send(Task {
                                msg,
                                chan: ack_ch.clone(),
                            });
                            Ok(())
                        })
                    })
                    .map_err(|e| error!("Consume failed: {}", e))
            }),
    )
}

struct Task {
    chan: channel::Channel<TcpStream>,
    msg: lapin_async::message::Delivery,
}

fn process_sync(from: crossbeam::Receiver<Task>) {
    while let Some(task) = from.recv() {
        info!("Processed msg '{}'", task.msg.delivery_tag);
        let _ = current_thread::block_on_all(
            task.chan
                .basic_ack(task.msg.delivery_tag, false)
                .map_err(|e| error!("Ack failed: {}", e)),
        );
    }
}
