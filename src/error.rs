use reqwest::{Response, StatusCode};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Reqwest had an Error {0}")]
    ReqwestError(reqwest::Error),
    #[error("Serde Json Parse Error {0}")]
    JSONError(serde_json::Error),
    #[error("Internal Error {0}")]
    Custom(String),
    #[error("A Bad Response Occurred")]
    BadResponse(Response),
    #[error("IO Error {0}")]
    IOError(std::io::Error),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::ReqwestError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IOError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::JSONError(err)
    }
}



pub fn from_error<E: std::error::Error>(e: E) -> Error {
    Error::Custom(e.to_string())
}



