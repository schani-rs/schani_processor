use std::path::PathBuf;
use std::process::Command;
use std::result::Result;

pub fn process_raw(path: &PathBuf, out: &PathBuf) -> Result<(), &'static str> {
    let output = try!(Command::new("rawtherapee")
                          .arg("-j90")
                          .arg("-Y")
                          .arg("-O")
                          .arg(out)
                          .arg("-c")
                          .arg(path)
                          .output()
                          .map_err(|_| "could not start rawtherapee"));

    println!("{:?}", output);

    if output.status.success() {
        Ok(())
    } else {
        Err("rawtherapee process returned error status code")
    }
}

#[cfg(test)]
mod test {
    use super::process_raw;
    use std::path::PathBuf;
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

        let path = PathBuf::from("resources/RAW1.NEF");
        let out = PathBuf::from("/tmp/");

        assert!(process_raw(&path, &out).is_ok());
    }

    #[test]
    fn process_nonexistent_file() {
        if !is_rawtherapee_installed() {
            print!("not testing RawTherapee because it's unavailable.");
            return ();
        }

        let path = PathBuf::from("resources/nope.NEF");
        let out = PathBuf::from("/tmp/");

        assert!(process_raw(&path, &out).is_err());
    }
}
