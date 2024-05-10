use reqwest::Response;
use serde::de::DeserializeOwned;
use tracing::debug;

use crate::InternalError;

pub mod query_string_builder;
pub mod time;

pub trait BetterResponseToJson {
    async fn better_to_json<D: DeserializeOwned>(self) -> Result<D, InternalError>;
}
impl BetterResponseToJson for Response {
    async fn better_to_json<D: DeserializeOwned>(self) -> Result<D, InternalError> {
        let response = self.text().await.map_err(InternalError::from)?;
        debug!(?response);
        let parsed: D = serde_json::from_str(&response).map_err(InternalError::from)?;
        Ok(parsed)
    }
}
