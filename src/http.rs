use crate::error;
use crate::error::Error;
use reqwest::header::{HeaderMap, HeaderName};
use reqwest::{Response, StatusCode};
use std::str::FromStr;

pub trait IntoResult {
    async fn into_result<E: ResponseError>(self) -> Result<Response, E>;
}
pub trait ResponseError {
    fn status_code(&self) -> StatusCode;

    async fn from_err(response: Response) -> Self;
}
impl IntoResult for Response {
    async fn into_result<E: ResponseError>(self) -> Result<Response, E> {
        if self.status().is_success() {
            return Ok(self);
        }
        let error = E::from_err(self).await;
        Err(error)
    }
}

pub fn get_header<T>(headers: &HeaderMap, header: HeaderName) -> Result<T, Error>
where
    T: FromStr,
    <T as FromStr>::Err: std::error::Error,
{
    headers
        .get(header.clone())
        .ok_or_else(|| Error::Custom(format!("Missing Header {}", header)))
        .and_then(|ct_len| ct_len.to_str().map_err(error::from_error))
        .and_then(|ct_len| ct_len.parse().map_err(error::from_error))
}
