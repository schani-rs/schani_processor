extern crate hyper;
#[macro_use]
extern crate log;
extern crate schani;
extern crate serde_json;
extern crate temporary;
extern crate url;

pub mod rawtherapee;
pub mod error;
mod store;

use rawtherapee::process_raw;
use store::{load_raw_file, upload_image_file};
use temporary::Directory;

pub fn process_raw_image(image_id: i32) -> Result<(), error::Error> {
    // Create a temporary directory.
    info!("processing image {} …", image_id);
    let directory = Directory::new("raw_images").unwrap();

    let tmp_path = directory.join(image_id.to_string() + &".NEF".to_string());
    println!("{:?}", tmp_path);
    let target_path = directory.into_path();

    try!(load_raw_file(image_id, &tmp_path));
    info!("loaded image {} …", image_id);
    try!(process_raw(&tmp_path, &target_path));
    info!("processed image {} …", image_id);
    try!(upload_image_file(image_id, &target_path));
    info!("uploaded image {} …", image_id);

    Ok(())
}
