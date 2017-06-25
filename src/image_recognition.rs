use super::error;
use std::fs::File;
use std::io::Read;
use std::io::prelude::*;
use std::path::PathBuf;

use hyper::{self, header, method, Url};
use hyper::client::Request;
use serde_json;

// TODO: create external library to prevent code duplication across microservices
#[derive(Debug, Deserialize)]
struct Prediction {
    class: String,
    score: f64,
}

pub fn classify_image(image_path: &PathBuf) -> Result<(), error::Error> {
    info!(
        "loading image file for processing from {:?}",
        image_path,
    );

    let mut image_buf = vec![];
    let mut image_file = try!(File::open(image_path));
    try!(image_file.read_to_end(&mut image_buf));
    info!("loaded image file, size is {} bytes", image_buf.len());

    let url = try!(Url::parse("http://image_recognition:8000/recognize"));
    let mut req = try!(Request::new(method::Method::Post, url));
    req.headers_mut().set(header::ContentLength(
        image_buf.len() as u64,
    ));
    let mut stream_req = try!(req.start());
    try!(stream_req.write_all(image_buf.as_slice()));
    try!(stream_req.flush());
    let resp = try!(stream_req.send());

    if resp.status != hyper::Ok {
        return Err(error::Error::Generic(format!(
            "unexpected status {} when sending image for recognition",
            resp.status
        )));
    }

    let predictions: Vec<Prediction> = try!(serde_json::from_reader(resp).map_err(|err| {
        error::Error::Generic(format!("could not deserialize predictions json: {}", err))
    }));

    info!("predicionts: {:?}", predictions);

    Ok(())
}
