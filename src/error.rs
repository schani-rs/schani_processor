use hyper::error::Error as HyperError;
use std::convert::From;
use std::error;
use std::fmt;
use std::io;
use url::ParseError;

#[derive(Debug)]
pub enum Error {
    Generic(String),
    HTTP(HyperError),
    IO(io::Error),
    URLError(ParseError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Generic(ref msg) => write!(f, "{}", msg),
            Error::HTTP(ref msg) => write!(f, "HTTP error: {}", msg),
            Error::IO(ref err) => write!(f, "IO error: {}", err),
            Error::URLError(ref err) => write!(f, "URL parsing error: {}", err),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Generic(ref msg) => &msg,
            Error::HTTP(ref err) => err.description(),
            Error::IO(ref err) => err.description(),
            Error::URLError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Generic(_) => None,
            Error::HTTP(ref err) => err.cause(),
            Error::IO(ref err) => err.cause(),
            Error::URLError(ref err) => err.cause(),
        }
    }
}

impl From<&'static str> for Error {
    fn from(orig: &str) -> Self {
        Error::Generic(orig.to_owned())
    }
}

impl From<ParseError> for Error {
    fn from(orig: ParseError) -> Self {
        Error::URLError(orig)
    }
}

impl From<HyperError> for Error {
    fn from(orig: HyperError) -> Self {
        Error::HTTP(orig)
    }
}

impl From<io::Error> for Error {
    fn from(orig: io::Error) -> Self {
        Error::IO(orig)
    }
}
