extern crate dotenv;
extern crate env_logger;
extern crate futures;
extern crate hyper;
extern crate lapin_async;
extern crate lapin_futures as lapin;
#[macro_use]
extern crate log;
extern crate resolve;
extern crate schani_library_client;
extern crate schani_store_client;
extern crate temporary;
extern crate tokio_core;
extern crate url;

// mod image_recognition;
mod error;
mod rawtherapee;
mod queue;

use std::env;
use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter};
use std::io::prelude::*;
use std::net;
use std::str;
use std::sync::Arc;

use futures::Future;
use hyper::Uri;
use schani_library_client::{Image, LibraryClient};
use schani_store_client::StoreClient;
use lapin_async::queue::Message;
use tokio_core::reactor::Core;

struct Config {
    amqp_addr: String,
    library_uri: Uri,
    store_uri: Uri,
}

fn get_config() -> Config {
    Config {
        amqp_addr: env::var("AMQP_ADDRESS").expect("AMQP_ADDRESS must be set"),
        library_uri: env::var("LIBRARY")
            .expect("LIBRARY must be set")
            .parse()
            .expect("could not parse library URI"),
        store_uri: env::var("STORE")
            .expect("STORE must be set")
            .parse()
            .expect("could not parse store URI"),
    }
}

pub fn run() {
    let config = get_config();

    // create the reactor
    let core = Core::new().unwrap();
    let host_addr = resolve::resolve_host(&config.amqp_addr)
        .expect("could not lookup host")
        .last()
        .unwrap();
    let addr = net::SocketAddr::new(host_addr, 5672);
    println!("connecting to AMQP service at {}", host_addr);
    let handle = core.handle();
    let lib_client = Arc::new(LibraryClient::new(config.library_uri.clone(), &handle));
    let store_client = Arc::new(StoreClient::new(config.store_uri.clone(), &handle));

    queue::run(&addr, core, &|message: Message| {
        info!("got message: {:?}", message);
        let image_id = str::from_utf8(&message.data)
            .unwrap()
            .to_string()
            .parse::<i32>()
            .unwrap();
        info!("image id: {:?}", image_id);

        let store_client2 = store_client.clone();
        let lib_client2 = lib_client.clone();
        Box::new(
            lib_client
                .get_image(image_id)
                .map_err(|_| io::Error::from(io::ErrorKind::Other))
                .and_then(move |image: Image| {
                    trace!("processing image {} …", image.id);

                    // Create a temporary directory.
                    futures::lazy(move || temporary::Directory::new("raw_images"))
                        .join(futures::future::ok(image))
                })
                .and_then(move |(directory, image)| {
                    let raw_path = directory.join(image.id.to_string() + &".NEF".to_string());
                    let img_path = directory.join(image.id.to_string() + &".jpg".to_string());
                    info!("temporary path: {:?}", raw_path);
                    info!("image path: {:?}", img_path);
                    let raw_id = image.raw_id.clone().unwrap();
                    let sidecar_id = image.sidecar_id.clone().unwrap();

                    store_client2
                        .get_raw_image(&raw_id)
                        .map_err(|_| io::Error::from(io::ErrorKind::Other))
                        .join(
                            store_client2
                                .get_sidecar(&sidecar_id)
                                .map_err(|_| io::Error::from(io::ErrorKind::Other)),
                        )
                        .and_then(move |(raw_data, sidecar)| {
                            info!(
                                        "loaded raw file {:?} ({} bytes) and sidecar ({} bytes) for image {}",
                                        raw_id,
                                        raw_data.len(),
                                        sidecar.len(),
                                        image.id
                                    );
                            let target_path = directory.into_path();
                            let file = File::create(&raw_path)?;
                            let mut buf_writer = BufWriter::new(file);
                            buf_writer.write_all(raw_data.as_slice())?;
                            info!("raw file written to disk");
                            let file = File::create(&img_path)?;
                            let mut buf_writer = BufWriter::new(file);
                            buf_writer.write_all(sidecar.as_slice())?;
                            info!("sidecar written to disk");

                            info!("start processing");
                            rawtherapee::process_raw(&raw_path, &target_path)?;

                            let file = File::open(&img_path)?;
                            let mut buf_reader = BufReader::new(file);
                            let mut buf = vec![];
                            buf_reader.read_to_end(&mut buf)?;
                            info!("processed JPEG is {} bytes big", buf.len());

                            Ok((image, buf, store_client2, lib_client2))
                        })
                        .and_then(|(mut image, buf, store_client2, lib_client)| {
                            store_client2
                                .upload_image(buf)
                                .map_err(|_| io::Error::from(io::ErrorKind::Other))
                                .and_then(move |jpeg_id| {
                                    // upload_image_file(image_id, &target_path)?;
                                    info!("uploaded image and got ID {}", jpeg_id);
                                    // update image
                                    info!(
                                        "updated image {} with new image id {}",
                                        image.id, jpeg_id
                                    );
                                    image.image_id = Some(jpeg_id);
                                    Ok((image, lib_client))
                                })
                        })
                        .and_then(|(image, lib_client)| {
                            lib_client
                                .update_image(image)
                                .map_err(|_| io::Error::from(io::ErrorKind::Other))
                        })
                        .map_err(|e| {
                            warn!("processing failed: {}", e);
                            e
                        })
                })
                .and_then(move |_| Ok(message)),
        )
    });
}
