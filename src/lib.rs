use std::{ops::Deref, sync::Arc};

use game_files::GameFilesAPIBuilder;
use reqwest::{header::ACCEPT, Client, RequestBuilder};
use serde::de::DeserializeOwned;
use tracing::{debug, trace};

pub use crate::error::Error;
use crate::http::IntoResult;

pub mod error;
pub mod game_files;
pub mod http;
pub(crate) mod mojang_time;
pub mod profile;
pub mod utils;
pub mod authentication {
    pub use minecraft_authentication::*;
}

#[derive(Clone, Debug)]
pub struct APIClient(pub(crate) Arc<InnerAPIClient>);
#[derive(Debug)]
pub struct InnerAPIClient {
    pub(crate) http_client: Client,
    pub(crate) game_files: GameFilesAPIBuilder,
}
impl APIClient {
    pub fn new(client: Client, game_files: GameFilesAPIBuilder) -> Self {
        Self(Arc::new(InnerAPIClient {
            http_client: client,
            game_files,
        }))
    }
}
impl Deref for APIClient {
    type Target = InnerAPIClient;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl APIClient {
    #[tracing::instrument]
    pub(crate) async fn process_json<D: DeserializeOwned>(
        &self,
        request: RequestBuilder,
    ) -> Result<D, Error> {
        let request = request.header(ACCEPT, "application/json");
        let request = request.build()?;
        let response = self.0.http_client.execute(request).await?;
        debug!(?response);
        let text = response.into_result::<Error>().await?.text().await?;
        trace!(?text);
        serde_json::from_str(&text).map_err(Error::from)
    }
}
#[cfg(test)]
pub(crate) mod test {

    use std::sync::Once;

    use reqwest::ClientBuilder;
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::{
        filter, fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
    };

    use crate::{game_files::GameFilesAPIBuilder, APIClient};

    static ENV_FILE: &str = "minecraft-rs.test.env";
    pub fn setup() -> APIClient {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            match dotenv::from_filename(ENV_FILE) {
                Ok(loaded) => {
                    println!("Loaded Dot Env from {loaded:?}");
                }
                Err(err) => {
                    println!("Could not load `{ENV_FILE}` {err}")
                }
            };

            if std::env::var("RUST_LOG").is_ok() {
                tracing_subscriber::registry()
                    .with(fmt::layer().pretty())
                    .with(EnvFilter::from_default_env())
                    .init();
            } else {
                let stdout_log = tracing_subscriber::fmt::layer().pretty();
                tracing_subscriber::registry()
                    .with(stdout_log.with_filter(filter::Targets::new().with_targets([
                        ("minecraft_rs", LevelFilter::DEBUG),
                        ("reqwest", LevelFilter::DEBUG),
                        ("tokio", LevelFilter::INFO),
                    ])))
                    .init();
            }
        });
        let user_agent =
            std::env::var("USER_AGENT").unwrap_or_else(|_| "minecraft-rs tester".to_owned());
        APIClient::new(
            ClientBuilder::new()
                .user_agent(user_agent)
                .build()
                .expect("Could not setup API Client"),
            GameFilesAPIBuilder::default(),
        )
    }
}
