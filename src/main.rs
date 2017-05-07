#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate schani;
extern crate schani_processor;
extern crate rocket;

use schani::images::RawtherapeeImage;
use schani_processor::process_raw;

#[post("/process/<file>")]
fn index(file: &str) -> &'static str {
    print!("procesing raw file {}", file);

    let image = RawtherapeeImage {
        name: "hello".to_string(),
        raw: "resources/".to_string() + &file,
        sidecar: "resources/DSC_2936.NEF.pp3".to_string(),
    };
    process_raw(&image);
    "file processed"
}

fn main() {
    rocket::ignite().mount("/", routes![index]).launch();
}
