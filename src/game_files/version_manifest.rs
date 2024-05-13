use chrono::{DateTime, Utc};
use reqwest::Url;
use serde::Deserialize;

use crate::{
    game_files::{release::data::ReleaseData, version_type::VersionType},
    mojang_time, APIClient, Error,
};

/// The main manliest file found at ['{launcher_meta}/mc/game/version_manifest_v2.json'](https://launchermeta.mojang.com/mc/game/version_manifest_v2.json)
#[derive(Deserialize, Debug)]
pub struct VersionManifest {
    pub latest: Latest,
    pub versions: Vec<Version>,
}
impl VersionManifest {
    pub fn get_version(&self, id: &str) -> Option<&Version> {
        self.versions.iter().find(|version| version.id.eq(id))
    }
    pub fn get_latest_snapshot(&self) -> &Version {
        self.versions
            .iter()
            .find(|version| version.id.eq(&self.latest.snapshot))
            .expect("The listed latest snapshot is not in the the version manifest?")
    }

    pub fn get_latest_release(&self) -> &Version {
        self.versions
            .iter()
            .find(|version| version.id.eq(&self.latest.release))
            .expect("The listed latest release is not in the the version manifest?")
    }
}
/// The latest version information provided by the api
#[derive(Deserialize, Debug)]
pub struct Latest {
    /// Latest Release
    pub release: String,
    /// Latest Snapshot
    pub snapshot: String,
}

/// The Version found within the manifest file.
/// Give basic information about the version
#[derive(Deserialize, Debug)]
pub struct Version {
    /// The Release Type
    #[serde(rename = "type")]
    pub release_type: VersionType,
    /// The URL to the release information
    pub url: String,
    /// The Release Name.
    pub id: String,
    #[serde(rename = "releaseTime", with = "mojang_time")]
    pub release_time: DateTime<Utc>,
    #[serde(with = "mojang_time")]
    pub time: DateTime<Utc>,
    pub sha1: String,
    #[serde(rename = "complianceLevel")]
    pub compliance_level: u8,
}

impl Version {
    /// Uses the url found inside the Version to pull the Release Data.
    /// ```
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = crate::test::setup();
    ///     let version_manifest = client.version_manifest().await?;
    ///     println!("Latest Release Info {:#?}", version_manifest.latest);
    ///     let snapshot = version_manifest
    ///         .get_latest_snapshot()
    ///         .get_release(&client)
    ///         .await?;
    ///     println!("Snapshot Release Info {:#?}", snapshot);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_release(&self, client: &APIClient) -> Result<ReleaseData, Error> {
        let url = Url::parse(&self.url).unwrap();
        let release = client
            .process_json::<ReleaseData>(client.http_client.get(url))
            .await?;
        Ok(release)
    }
}
