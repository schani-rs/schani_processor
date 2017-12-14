use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::io::prelude::*;
use std::path::PathBuf;

use futures::{Future, Map, Stream};
use futures::future::{err, ok, AndThen, IntoFuture};
use hyper::{self, header};
use hyper::client::{Client, FutureResponse, Request, Response};
use serde_json::{self, Value};
use tokio_core::reactor::Handle;

use error;

fn get_raw_file_id<F>(
    handle: &Handle,
    image_id: i32,
) -> Box<Future<Item = i32, Error = error::Error>> {
    let client = Client::new(handle);
    let url = format!("http://store:8000/api/images/{}", image_id)
        .parse()
        .map_err(|e| {
            error::Error::Generic(format!("could not parse uri: {}", e))
        })
        .unwrap();
    let work = client.get(url).and_then(|resp| {
        /*if resp.status() != hyper::Ok {
            return err(error::Error::Generic(format!(
                "got unexpected HTTP status {} when loading image data",
                resp.status()
            )));
        }*/

        resp.body().concat2().and_then(move |body| {
            let json: Value = serde_json::from_slice(&body).map_err(|e| {
                error::Error::Generic(format!("could not parse image json: {}", e.description()))
            }).unwrap();
            info!("got image data as json: {:?}", json);
            Ok(json["raw_image_id"].as_u64().unwrap() as i32)
        })
    }).map_err(|e| {
        error::Error::HTTP(e)
    });

    Box::new(work)
}

/*
pub fn load_raw_file(image_id: i32, target_path: &PathBuf) -> Result<(), error::Error> {
    let raw_file_id = try!(get_raw_file_id(image_id));
    let client = Client::new();
    let url = Url::parse(&format!(
        "http://store:8000/api/raw_images/{}/file",
        raw_file_id
    ))?;
    let mut resp: Response = try!(client.get(url).send());

    let mut buf = vec![];
    resp.read_to_end(&mut buf)?;

    info!("downloaded raw file, size: {} bytes", buf.len());

    let mut f = File::create(target_path)?;
    f.write_all(buf.as_slice())?;

    Ok(())
}

pub fn upload_image_file(image_id: i32, output_path: &PathBuf) -> Result<(), error::Error> {
    let image_path = output_path.join(image_id.to_string() + &".jpg".to_string());
    let sidecar_path = output_path.join(image_id.to_string() + &".pp3".to_string());
    info!(
        "loading image file from {:?} and sidecar file from {:?}",
        image_path,
        sidecar_path
    );

    let mut image_buf = vec![];
    let mut image_file = try!(File::open(image_path));
    image_file.read_to_end(&mut image_buf)?;
    info!("loaded image file, size is {} bytes", image_buf.len());

    let url = Url::parse(&format!("http://store:8000/api/images/{}/file", image_id))?;
    let mut req = Request::new(method::Method::Post, url)?;
    req.headers_mut()
        .set(header::ContentLength(image_buf.len() as u64));
    let mut stream_req = req.start()?;
    stream_req.write_all(image_buf.as_slice())?;
    stream_req.flush()?;
    stream_req.send()?;

    Ok(())
}

*/
