extern crate dotenv;
extern crate env_logger;
extern crate futures;
extern crate hyper;
extern crate lapin_async;
extern crate lapin_futures as lapin;
#[macro_use]
extern crate log;
extern crate resolve;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate temporary;
extern crate tokio_core;
extern crate url;

// mod image_recognition;
mod error;
mod rawtherapee;
mod service;
mod queue;

use std::env;
use std::net;
use std::str;

use lapin_async::queue::Message;
use tokio_core::reactor::Core;

pub fn run() {
    // create the reactor
    let core = Core::new().unwrap();
    let host = env::var("AMQP_ADDRESS").expect("AMQP_ADDRESS must be set");
    let host_addr = resolve::resolve_host(&host)
        .expect("could not lookup host")
        .last()
        .unwrap();
    let addr = net::SocketAddr::new(host_addr, 5672);
    println!("connecting to AMQP service at {}", host_addr);

    queue::run(&addr, core, &|message: &Message| {
        info!("got message: {:?}", message);
        let image_id = str::from_utf8(&message.data).unwrap().to_string();
        info!("image id: {:?}", image_id);
        service::processor::process_raw_image(image_id).map_err(|err| err.into())
    });
}
