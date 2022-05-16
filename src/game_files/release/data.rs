use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt;


use chrono::{DateTime, Utc};
use serde::de::{Error, MapAccess, Visitor};
use serde::{de, Deserialize, Deserializer};
use serde_json::Value;
use std::str::FromStr;
use crate::game_files::release::argument::Arguments;
use crate::game_files::release::library::Library;
use crate::game_files::version_type::VersionType;
use crate::mojang_time;

#[derive(Deserialize, Debug)]
pub struct ReleaseData {
    pub downloads: HashMap<String, Download>,
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
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

#[derive(Deserialize, Debug)]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u32,
    pub total_size: Option<u32>,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct Download {
    pub sha1: String,
    pub size: u32,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u32,
}

#[derive(Deserialize, Debug)]
pub struct Logging {
    pub argument: String,
    pub file: LoggingFile,
    #[serde(rename = "type")]
    pub logging_type: String,
}

#[derive(Deserialize, Debug)]
pub struct LoggingFile {
    pub id: String,
    pub sha1: String,
    pub size: u32,
    pub url: String,
}
