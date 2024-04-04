use std::ops::Deref;

pub use crate::error::Error;
use crate::http::IntoResult;
use game_files::GameFilesAPIBuilder;
pub use microsoft_authentication::*;
use reqwest::header::ACCEPT;
use reqwest::RequestBuilder;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub mod error;
pub mod game_files;
pub mod http;
pub mod microsoft_authentication;
pub mod mojang_api;
pub(crate) mod mojang_time;
pub mod profile;
pub mod utils;
#[derive(Clone, Debug, Deserialize, Serialize, Default)]

pub struct APIClientSettings {
    #[serde(default)]
    pub game_files: GameFilesAPIBuilder,
    pub auth_properties: Option<AuthProperties>,
}
pub struct APIClient {
    pub(crate) http_client: reqwest::Client,
    pub(crate) game_files: GameFilesAPIBuilder,
}

pub struct APIClientWithAuth {
    pub api_client: APIClient,
    pub auth_properties: AuthProperties,
}
impl Deref for APIClientWithAuth {
    type Target = APIClient;

    fn deref(&self) -> &Self::Target {
        &self.api_client
    }
}

impl APIClient {
    pub(crate) async fn process_json<D: DeserializeOwned>(
        &self,
        request: RequestBuilder,
    ) -> Result<D, Error> {
        let request = request.header(ACCEPT, "application/json");
        let request = request.build()?;
        let response = self.http_client.execute(request).await?.into_result()?;
        response.json().await.map_err(Error::from)
    }
}
