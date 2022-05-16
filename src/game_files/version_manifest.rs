use crate::Error;
use crate::{APIClient};
use chrono::{DateTime, Utc};
use reqwest::Url;
use serde::Deserialize;
use crate::game_files::release::data::ReleaseData;
use crate::game_files::release::Release;
use crate::game_files::version_type::VersionType;
use crate::mojang_time;

/// The main manliest file found at ['{launcher_meta}/mc/game/version_manifest_v2.json'](https://launchermeta.mojang.com/mc/game/version_manifest_v2.json)
#[derive(Deserialize, Debug)]
pub struct VersionManifest {
    pub latest: Latest,
    pub versions: Vec<Version>,
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
    pub async fn get_release<'a>(&self, client: &'a APIClient) -> Result<Release<'a>, Error> {
        let url = Url::parse(&self.url).unwrap();
        let release = client.process_json::<ReleaseData>(client.http_client.get(url)).await?;
        Ok(Release {
            client,
            data: release,
        })
    }
}
