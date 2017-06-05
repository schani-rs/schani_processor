use schani::images::RawtherapeeImage;
use std::process::Command;
use std::result::Result;

pub fn process_raw(img: &RawtherapeeImage) -> Result<(), &'static str> {
    let status = try!(Command::new("rawtherapee-cli")
                          .arg("-j90")
                          .arg("-Y")
                          .arg("-c")
                          .arg(&img.raw)
                          .status()
                          .map_err(|_| "could not process raw image"));

    if status.success() {
        Ok(())
    } else {
        Err("rawtherapee process returned error status code")
    }
}

#[cfg(test)]
mod test {
    use super::super::schani::images::RawtherapeeImage;
    use super::process_raw;
    use std::process::Command;

    fn is_rawtherapee_installed() -> bool {
        Command::new("rawtherapee-cli").status().is_ok()
    }

    #[test]
    fn process_sample_image() {
        if !is_rawtherapee_installed() {
            print!("not testing RawTherapee because it's unavailable.");
            return ();
        }

        let raw1 = RawtherapeeImage {
            name: "raw1".to_string(),
            raw: "resources/RAW1.NEF".to_string(),
            sidecar: "resources/RAW1.NEF.pp3".to_string(),
        };

        assert!(process_raw(&raw1).is_ok());
    }

    #[test]
    fn process_nonexistent_file() {
        if !is_rawtherapee_installed() {
            print!("not testing RawTherapee because it's unavailable.");
            return ();
        }

        let raw1 = RawtherapeeImage {
            name: "x".to_string(),
            raw: "resources/x".to_string(),
            sidecar: "resources/x".to_string(),
        };

        assert!(process_raw(&raw1).is_err());
    }
}
