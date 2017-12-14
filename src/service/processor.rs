use error;

use temporary;
use rawtherapee;

// use store::{load_raw_file, upload_image_file};
// use image_recognition::classify_image;

pub fn process_raw_image(image_id: String) -> Result<(), error::Error> {
    // Create a temporary directory.
    info!("processing image {} …", image_id);
    let directory = temporary::Directory::new("raw_images").unwrap();

    let tmp_path = directory.join(image_id.to_string() + &".NEF".to_string());
    println!("{:?}", tmp_path);
    let img_path = directory.join(image_id.to_string() + &".jpg".to_string());
    let target_path = directory.into_path();

    // load_raw_file(image_id, &tmp_path)?;
    info!("loaded image {} …", image_id);
    rawtherapee::process_raw(&tmp_path, &target_path)?;
    info!("processed image {} …", image_id);
    // upload_image_file(image_id, &target_path)?;
    info!("uploaded image {} …", image_id);

    Ok(())
}
