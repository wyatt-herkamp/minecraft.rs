use reqwest::{Response, StatusCode};
use thiserror::Error;

use crate::http::ResponseError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Reqwest had an Error {0}")]
    ReqwestError(reqwest::Error),
    #[error("Serde Json Parse Error {0}")]
    JSONError(#[from] serde_json::Error),
    #[error("Internal Error {0}")]
    Custom(String),
    #[error("A Bad Response Occurred")]
    BadResponse(Response),
    #[error("IO Error {0}")]
    IOError(#[from] std::io::Error),
    #[error("Authorization Not Configured")]
    AuthorizationNotConfigured,
    #[error("Invalid URL: {0}")]
    URLParse(#[from] url::ParseError),
}
impl ResponseError for Error {
    fn status_code(&self) -> reqwest::StatusCode {
        if let Error::BadResponse(response) = &self {
            return response.status();
        }
        StatusCode::default()
    }

    async fn from_err(response: Response) -> Self {
        Error::BadResponse(response)
    }
}
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::ReqwestError(err)
    }
}
pub fn from_error<E: std::error::Error>(e: E) -> Error {
    Error::Custom(e.to_string())
}
