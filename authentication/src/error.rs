use std::{error::Error, fmt::Debug};

use reqwest::{Response, StatusCode};
use thiserror::Error;
use tracing::{debug, warn};
pub trait IntoResult {
    async fn into_result<E: ResponseError>(self) -> Result<Result<Response, E>, InternalError>;
}
pub trait ResponseError: Debug {
    fn status_code(&self) -> StatusCode;

    async fn from_err(response: Response) -> Result<Self, InternalError>
    where
        Self: Sized;
}

impl IntoResult for Response {
    async fn into_result<E: ResponseError>(self) -> Result<Result<Response, E>, InternalError> {
        if self.status().is_success() {
            debug!(?self, "Successful Response");
            return Ok(Ok(self));
        }
        if self.status().is_redirection() {
            warn!(
                ?self,
                "No Redirections should happen. Please Report this error"
            );
        }

        debug!(?self, "Bad Response");
        if self.status().is_server_error() {
            return Err(InternalError::BadResponse(self));
        }
        let error = E::from_err(self).await?;
        debug!(?error, "Parsed Error");
        Ok(Err(error))
    }
}
#[derive(Debug, Error)]
pub enum BadResponseOrError<E: Error + ResponseError> {
    #[error(transparent)]
    Error(#[from] InternalError),
    #[error(transparent)]
    ResponseError(#[from] E),
}
#[derive(Error, Debug)]
pub enum InternalError {
    #[error("Reqwest had an Error {0}")]
    ReqwestError(reqwest::Error),
    #[error("Serde Json Parse Error {0}")]
    JSONError(#[from] serde_json::Error),
    #[error("A Bad Response Occurred")]
    BadResponse(Response),
    #[error("IO Error {0}")]
    IOError(#[from] std::io::Error),
    #[error("Invalid URL: {0}")]
    URLParse(#[from] url::ParseError),
}

impl From<reqwest::Error> for InternalError {
    fn from(err: reqwest::Error) -> InternalError {
        InternalError::ReqwestError(err)
    }
}
