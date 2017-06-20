extern crate amq_protocol;
extern crate env_logger;
extern crate dotenv;
extern crate futures;
extern crate lapin_futures as lapin;
extern crate resolve;
extern crate schani_processor;
extern crate tokio_core;

use std::env;
use std::net;
use amq_protocol::types::FieldTable;
use dotenv::dotenv;
use futures::Stream;
use futures::future::Future;
use lapin::client::ConnectionOptions;
use lapin::channel::{BasicConsumeOptions, QueueDeclareOptions};
use schani_processor::process_raw_image;
use tokio_core::reactor::Core;
use tokio_core::net::TcpStream;
use resolve::resolve_host;

fn main() {
    env_logger::init().unwrap();
    dotenv().ok();

    // create the reactor
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let host = env::var("AMQP_ADDRESS").expect("AMQP_ADDRESS must be set");
    let host_addr = resolve_host(&host)
        .expect("could not lookup host")
        .last()
        .unwrap();
    let addr = net::SocketAddr::new(host_addr, 5672);

    println!("connecting to AMQP service at {}", host_addr);
    core.run(
        TcpStream::connect(&addr, &handle)
            .and_then(|stream| {

                // connect() returns a future of an AMQP Client
                // that resolves once the handshake is done
                lapin::client::Client::connect(stream, &ConnectionOptions::default())
            })
            .and_then(|client| {

                // create_channel returns a future that is resolved
                // once the channel is successfully created
                client.create_channel()
            })
            .and_then(|channel| {
                let id = channel.id;
                println!("created channel with id: {}", id);

                let ch = channel.clone();
                channel
                    .queue_declare("raw", &QueueDeclareOptions::default(), FieldTable::new())
                    .and_then(move |_| {
                        println!("channel {} declared queue {}", id, "raw");

                        // basic_consume returns a future of a message
                        // stream. Any time a message arrives for this consumer,
                        // the for_each method would be called
                        channel.basic_consume(
                            "raw",
                            "raw_processor",
                            &BasicConsumeOptions::default(),
                        )
                    })
                    .and_then(|stream| {
                        println!("got consumer stream");

                        stream.for_each(move |message| {
                            let file_id_str = std::str::from_utf8(&message.data).unwrap();
                            let file_id = file_id_str.parse::<i32>().unwrap();
                            println!("got message: {:?}", message);
                            println!("file id: {:?}", file_id);
                            try!(process_raw_image(file_id).map_err(|err| {
                                std::io::Error::new(std::io::ErrorKind::Other, err)
                            }));
                            ch.basic_ack(message.delivery_tag);
                            Ok(())
                        })
                    })
            }),
    ).unwrap();
}
