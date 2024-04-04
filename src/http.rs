use crate::error;
use crate::error::Error;
use reqwest::header::{HeaderMap, HeaderName};
use reqwest::Response;
use std::str::FromStr;

pub trait IntoResult {
    fn into_result(self) -> Result<Response, Error>;
}

impl IntoResult for Response {
    fn into_result(self) -> Result<Response, Error> {
        if self.status().is_success() {
            Ok(self)
        } else {
            Err(Error::BadResponse(self))
        }
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
