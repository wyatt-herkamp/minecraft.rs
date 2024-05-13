use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    game_files::{
        release::{argument::Arguments, library::Library},
        version_type::VersionType,
    },
    mojang_time,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseData {
    pub downloads: Downloads,
    pub arguments: Option<Arguments>,
    #[serde(default)]
    pub logging: HashMap<String, Logging>,
    pub libraries: Vec<Library>,
    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndex,
    pub assets: String,
    #[serde(rename = "minimumLauncherVersion")]
    pub minimum_launcher_version: u64,
    /// The Release Type
    #[serde(rename = "type")]
    pub release_type: VersionType,
    /// The Release Name.
    pub id: String,
    #[serde(rename = "releaseTime", with = "mojang_time")]
    pub release_time: DateTime<Utc>,
    #[serde(with = "mojang_time")]
    pub time: DateTime<Utc>,
    #[serde(rename = "complianceLevel")]
    pub compliance_level: u8,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(rename = "javaVersion")]
    pub java_version: JavaVersion,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}
impl ReleaseData {
    pub fn has_server(&self) -> bool {
        self.downloads.server.is_some()
    }
    pub fn has_mappings(&self) -> bool {
        self.downloads.server_mapping.is_some() && self.downloads.client_mapping.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub total_size: Option<u64>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Download {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Downloads {
    pub client: Download,
    pub server: Option<Download>,
    pub server_mapping: Option<Download>,
    pub client_mapping: Option<Download>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Logging {
    pub argument: String,
    pub file: LoggingFile,
    #[serde(rename = "type")]
    pub logging_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingFile {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}
