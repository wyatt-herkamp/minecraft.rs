pub mod assets;
pub mod release;
pub mod version_manifest;
pub mod version_type;

use std::borrow::Cow;

use reqwest::Url;
use serde::{Deserialize, Serialize};

use self::version_manifest::VersionManifest;
use crate::{APIClient, Error};
pub static RESOURCE_URL_BASE: &str = "https://resources.download.minecraft.net";
pub static LIBRARY_URL_BASE: &str = "https://libraries.minecraft.net";
pub static LAUNCHER_META_URL_BASE: &str = "https://piston-meta.mojang.com";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GameFilesAPIBuilder {
    pub resource_base: Cow<'static, str>,
    pub library_base: Cow<'static, str>,
    pub launcher_meta: Cow<'static, str>,
}

impl Default for GameFilesAPIBuilder {
    fn default() -> Self {
        GameFilesAPIBuilder {
            resource_base: Cow::Borrowed(RESOURCE_URL_BASE),
            library_base: Cow::Borrowed(LIBRARY_URL_BASE),
            launcher_meta: Cow::Borrowed(LAUNCHER_META_URL_BASE),
        }
    }
}

impl APIClient {
    //TODO include access point for Game File API
    pub async fn version_manifest(&self) -> Result<VersionManifest, Error> {
        self.process_json(
            self.http_client.get(
                self.game_files
                    .create_launcher_url("mc/game/version_manifest_v2.json"),
            ),
        )
        .await
    }
}

impl GameFilesAPIBuilder {
    /// Generates a Resource URL from the path given
    pub fn create_resource_url<S: AsRef<str>>(&self, s: S) -> Url {
        Url::parse(&format!("{}/{}", &self.resource_base, s.as_ref())).unwrap()
    }
    /// Generates a Library URL from the path given
    pub fn create_library_url<S: AsRef<str>>(&self, s: S) -> Url {
        Url::parse(&format!("{}/{}", &self.library_base, s.as_ref())).unwrap()
    }
    /// Generates a Launcher URL from the path given
    pub fn create_launcher_url<S: AsRef<str>>(&self, s: S) -> Url {
        Url::parse(&format!("{}/{}", &self.launcher_meta, s.as_ref())).unwrap()
    }
}
#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::time::sleep;

    #[tokio::test]
    async fn version_manifest_v2() -> anyhow::Result<()> {
        let client = crate::test::setup();
        let version_manifest = client.version_manifest().await?;
        println!("Latest Release Info {:#?}", version_manifest.latest);
        let version = version_manifest.get_version("1.20.6");
        assert!(version.is_some(), "Could not find version 1.20.6");
        println!("Version 1.20.6 {:#?}", version);
        Ok(())
    }

    #[tokio::test]
    async fn parse_version_latest_releases() -> anyhow::Result<()> {
        let client = crate::test::setup();
        let version_manifest = client.version_manifest().await?;
        println!("Latest Release Info {:#?}", version_manifest.latest);
        let snapshot = version_manifest
            .get_latest_snapshot()
            .get_release(&client)
            .await?;
        println!("Snapshot Release Info {:#?}", snapshot);
        let release = version_manifest
            .get_latest_release()
            .get_release(&client)
            .await?;
        println!("Stable Release Info {:#?}", release);

        Ok(())
    }
    #[ignore = "We will be making a lot of requests to Mojank"]
    #[tokio::test]
    async fn parse_version_all_releases() -> anyhow::Result<()> {
        let client = crate::test::setup();
        let version_manifest = client.version_manifest().await?;
        println!("Latest Release Info {:#?}", version_manifest.latest);
        for version in &version_manifest.versions {
            match version.get_release(&client).await {
                Ok(ok) => {
                    println!("Version Parsed {ok:#?}")
                }
                Err(err) => {
                    eprintln!(
                        "Could not read release data from {:?} \n Error \n {err:#?}",
                        version
                    );
                }
            }
            sleep(Duration::from_secs(1)).await
        }

        Ok(())
    }
}
