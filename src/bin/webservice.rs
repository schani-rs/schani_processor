#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate schani;
extern crate schani_processor;
extern crate rocket;

use rocket::http::Status;
use rocket::response::status::Custom;
use std::result::Result;
use schani::images::RawtherapeeImage;
use schani_processor::rawtherapee::process_raw;

#[post("/process/<file>")]
fn index(file: &str) -> Result<&'static str, Custom<&'static str>> {
    print!("procesing raw file {}", file);

    let image = RawtherapeeImage {
        name: "hello".to_string(),
        raw: "resources/".to_string() + &file,
        sidecar: "resources/DSC_2936.NEF.pp3".to_string(),
    };

    process_raw(&image)
        .map(|_| "file processed")
        .map_err(|err| Custom(Status::InternalServerError, err))
}

fn main() {
    rocket::ignite().mount("/", routes![index]).launch();
}
