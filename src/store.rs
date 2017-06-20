use std::fs::File;
use std::io::Read;
use std::io::prelude::*;
use std::path::PathBuf;

use hyper::{self, header, method, Url};
use hyper::client::{Client, Request, Response};
use serde_json::{self, Value};
use serde_urlencoded;

pub fn load_raw_file(image_id: i32, target_path: &PathBuf) -> Result<(), super::error::Error> {
    let client = Client::new();
    let url = try!(Url::parse(&format!(
        "http://store:8000/api/raw_images/{}/file",
        image_id.to_string()
    )));
    let mut resp: Response = try!(client.get(url).send());

    let mut buf = vec![];
    try!(resp.read_to_end(&mut buf));

    println!("downloaded raw file size: {} bytes", buf.len());

    let mut f = try!(File::create(target_path));
    try!(f.write_all(buf.as_slice()));

    Ok(())
}

fn build_new_image_body(
    title: String,
    description: String,
    license: String,
    side_car_file: String,
    raw_image_id: i32,
) -> String {
    serde_urlencoded::to_string(
        [
            ("title", title.to_owned()),
            ("description", description.to_owned()),
            ("license", license.to_owned()),
            ("side_car_file", side_car_file.to_owned()),
            ("raw_image_id", raw_image_id.to_string()),
        ],
    ).unwrap()
}

fn save_image_data(
    title: String,
    description: String,
    license: String,
    side_car_file: String,
    raw_image_id: i32,
) -> Result<u64, super::error::Error> {
    let client = Client::new();
    let mut resp: hyper::client::Response = try!(
        client
            .post("http://store:8000/api/images")
            .header(hyper::header::ContentType::form_url_encoded())
            .body(&build_new_image_body(
                title,
                description,
                license,
                side_car_file,
                raw_image_id,
            ))
            .send()
    );
    if resp.status != hyper::status::StatusCode::Created {
        return Err(super::error::Error::Generic(format!(
            "unexpected HTTP status {} when sending image data to store",
            resp.status
        )));
    }

    let mut resp_text = String::new();
    try!(resp.read_to_string(&mut resp_text));
    let resp_json: Value = try!(serde_json::from_str(&resp_text).map_err(|err| {
        super::error::Error::Generic(format!("could not read response JSON: {}", err))
    }));
    let id = resp_json["id"].as_u64().unwrap();
    info!("saved image to store and got id {}", id);

    Ok(id)
}

pub fn upload_image_file(
    raw_image_id: i32,
    output_path: &PathBuf,
) -> Result<(), super::error::Error> {
    let image_path = output_path.join(raw_image_id.to_string() + &".jpg".to_string());
    let sidecar_path = output_path.join(raw_image_id.to_string() + &".pp3".to_string());
    info!(
        "loading image file from {:?} and sidecar file from {:?}",
        image_path,
        sidecar_path
    );

    let mut image_buf = vec![];
    let mut image_file = try!(File::open(image_path));
    try!(image_file.read_to_end(&mut image_buf));
    info!("loaded image file, size is {} bytes", image_buf.len());

    let image_id = try!(save_image_data(
        "test".to_string(),
        "descr".to_string(),
        "CC".to_string(),
        "non".to_string(),
        raw_image_id,
    ));

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
