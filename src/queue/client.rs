use std::io;
use std::net;
use std::thread;

use futures::{Future, Stream};
use lapin;
use lapin::message::Delivery;
use lapin::client::ConnectionOptions;
use lapin::types::FieldTable;
use lapin::channel::{BasicConsumeOptions, QueueDeclareOptions};
use tokio_core::reactor::Core;
use tokio_core::net::TcpStream;

pub fn run(
    addr: &net::SocketAddr,
    mut core: Core,
    fun: &Fn(Delivery) -> Box<Future<Item = Delivery, Error = io::Error>>,
) {
    let work = TcpStream::connect(addr, &core.handle())
        .and_then(|stream| {
            // connect() returns a future of an AMQP Client
            // that resolves once the handshake is done
            lapin::client::Client::connect(stream, &ConnectionOptions::default())
        })
        .and_then(|(client, heartbeat)| {
            let heartbeat_client = client.clone();
            thread::Builder::new()
                .name("heartbeat thread".to_string())
                .spawn(move || {
                    Core::new()
                        .unwrap()
                        .run(heartbeat(&heartbeat_client))
                        .unwrap();
                })
                .unwrap();
            // create_channel returns a future that is resolved
            // once the channel is successfully created
            client.create_channel()
        })
        .and_then(|channel| {
            let id = channel.id;
            println!("created channel with id: {}", id);

            let ch = channel.clone();
            channel
                .queue_declare("raw", &QueueDeclareOptions::default(), &FieldTable::new())
                .and_then(move |_| {
                    println!("channel {} declared queue {}", id, "raw");

                    // basic_consume returns a future of a message
                    // stream. Any time a message arrives for this consumer,
                    // the for_each method would be called
                    channel.basic_consume(
                        "raw",
                        "raw_processor",
                        &BasicConsumeOptions::default(),
                        &FieldTable::new(),
                    )
                })
                .and_then(|stream| {
                    println!("got consumer stream");

                    stream.for_each(move |message| {
                        let chan = ch.clone();
                        fun(message).and_then(move |message| {
                            chan.basic_ack(message.delivery_tag);
                            Ok(())
                        })
                    })
                })
        });

    core.run(work).unwrap();
}
