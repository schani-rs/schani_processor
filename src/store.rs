use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::io::prelude::*;
use std::path::PathBuf;

use hyper::{self, header, method, Url};
use hyper::client::{Client, Request, Response};
use serde_json::{self, Value};

fn get_raw_file_id(image_id: i32) -> Result<i32, super::error::Error> {
    let client = Client::new();
    let url = try!(Url::parse(
        &format!("http://store:8000/api/images/{}", image_id),
    ));
    let resp: Response = try!(client.get(url).send());
    if resp.status != hyper::Ok {
        return Err(super::error::Error::Generic(format!(
            "got unexpected HTTP status {} when loading image data",
            resp.status
        )));
    }

    let json: Value = try!(serde_json::from_reader(resp).map_err(|e| {
        super::error::Error::Generic(format!("could not parse image json: {}", e.description()))
    }));
    info!("got image data as json: {:?}", json);
    Ok(json["raw_image_id"].as_u64().unwrap() as i32)
}

pub fn load_raw_file(image_id: i32, target_path: &PathBuf) -> Result<(), super::error::Error> {
    let raw_file_id = try!(get_raw_file_id(image_id));
    let client = Client::new();
    let url = try!(Url::parse(&format!(
        "http://store:8000/api/raw_images/{}/file",
        raw_file_id
    )));
    let mut resp: Response = try!(client.get(url).send());

    let mut buf = vec![];
    try!(resp.read_to_end(&mut buf));

    info!("downloaded raw file, size: {} bytes", buf.len());

    let mut f = try!(File::create(target_path));
    try!(f.write_all(buf.as_slice()));

    Ok(())
}

pub fn upload_image_file(image_id: i32, output_path: &PathBuf) -> Result<(), super::error::Error> {
    let image_path = output_path.join(image_id.to_string() + &".jpg".to_string());
    let sidecar_path = output_path.join(image_id.to_string() + &".pp3".to_string());
    info!(
        "loading image file from {:?} and sidecar file from {:?}",
        image_path,
        sidecar_path
    );

    let mut image_buf = vec![];
    let mut image_file = try!(File::open(image_path));
    try!(image_file.read_to_end(&mut image_buf));
    info!("loaded image file, size is {} bytes", image_buf.len());

    let url = try!(Url::parse(
        &format!("http://store:8000/api/images/{}/file", image_id),
    ));
    let mut req = try!(Request::new(method::Method::Post, url));
    req.headers_mut().set(header::ContentLength(
        image_buf.len() as u64,
    ));
    let mut stream_req = try!(req.start());
    try!(stream_req.write_all(image_buf.as_slice()));
    try!(stream_req.flush());
    try!(stream_req.send());

    Ok(())
}
