extern crate amq_protocol;
extern crate futures;
extern crate tokio_core;
extern crate lapin_futures as lapin;

use amq_protocol::types::FieldTable;
use futures::Stream;
use futures::future::Future;
use tokio_core::reactor::Core;
use tokio_core::net::TcpStream;
use lapin::client::ConnectionOptions;
use lapin::channel::{BasicConsumeOptions, QueueDeclareOptions};

fn main() {

    // create the reactor
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let addr = "127.0.0.1:5672".parse().unwrap();

    core.run(TcpStream::connect(&addr, &handle)
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
                    println!("channel {} declared queue {}", id, "hello");

                    // basic_consume returns a future of a message
                    // stream. Any time a message arrives for this consumer,
                    // the for_each method would be called
                    channel.basic_consume("raw", "raw_processor", &BasicConsumeOptions::default())
                })
                .and_then(|stream| {
                    println!("got consumer stream");

                    stream.for_each(move |message| {
                                        println!("got message: {:?}", message);
                                        println!("decoded message: {:?}",
                                                 std::str::from_utf8(&message.data).unwrap());
                                        ch.basic_ack(message.delivery_tag);
                                        Ok(())
                                    })
                })
        }))
        .unwrap();
}