pub mod account;
pub mod microsoft;
pub mod minecraft;
pub mod xbox;

use std::{ops::Deref, sync::Arc};

pub use account::*;
pub use microsoft::*;
pub use minecraft::*;
use reqwest::{header::ACCEPT, Client, RequestBuilder};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use utils::BetterResponseToJson;
pub use xbox::*;
pub mod error;
pub use crate::error::InternalError;
use tracing::error;
pub(crate) mod utils;
#[derive(Clone, Debug)]
pub struct AuthenticationClient(pub(crate) Arc<InnerAuthenticationClient>);
impl AuthenticationClient {
    pub fn new(client: Client, auth_properties: impl Into<AuthProperties>) -> Self {
        Self(Arc::new(InnerAuthenticationClient {
            http_client: client,
            auth_properties: auth_properties.into(),
        }))
    }
    pub(crate) async fn process_json<D: DeserializeOwned>(
        &self,
        request: RequestBuilder,
    ) -> Result<D, InternalError> {
        let request = request.header(ACCEPT, "application/json");
        let request = request.build()?;
        let response = self.0.http_client.execute(request).await?;
        if !response.status().is_success() {
            error!(?response, "Could not process request");
            let body = response.text().await?;
            error!(?body, "Body Of Response");
            return Err(InternalError::IOError(std::io::Error::new(
                std::io::ErrorKind::Other,
                "PLACE HOLDER",
            )));
        }
        response.better_to_json().await.map_err(InternalError::from)
    }
}
impl Deref for AuthenticationClient {
    type Target = InnerAuthenticationClient;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}
#[derive(Debug)]
pub struct InnerAuthenticationClient {
    pub http_client: Client,
    pub auth_properties: AuthProperties,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthProperties {
    /// Microsoft Client ID
    pub azura_microsoft_client: String,
}
