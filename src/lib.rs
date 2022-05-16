use reqwest::header::ACCEPT;
use reqwest::RequestBuilder;
use serde::de::DeserializeOwned;
pub use crate::error::Error;
use crate::http::IntoResult;


pub mod error;
pub mod http;
pub mod utils;
pub mod mojang_time;
#[cfg(feature = "profile_api")]
pub mod profile;
#[cfg(feature = "game_files")]
pub mod game_files;
#[cfg(feature = "mojang_api")]
pub mod mojang_api;
#[cfg(feature = "microsoft_authentication")]
pub mod microsoft_authentication;

pub struct APIClient {
    pub(crate) http_client: reqwest::Client,
    #[cfg(feature = "game_files")]
    pub game_files: game_files::GameFilesAPIBuilder,
    #[cfg(feature = "microsoft_authentication")]
    pub auth_properties: microsoft_authentication::AuthProperties,

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