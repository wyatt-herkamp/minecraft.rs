use serde::Deserialize;
use std::fmt::{Display, Formatter};

#[derive(Deserialize, Debug)]
pub enum VersionType {
    #[serde(rename = "snapshot")]
    Snapshot,
    #[serde(rename = "release")]
    Release,
    #[serde(rename = "old_beta")]
    OldBeta,
    #[serde(rename = "old_alpha")]
    OldAlpha,
}

impl Display for VersionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let version_name = match self {
            VersionType::Snapshot => "snapshot",
            VersionType::Release => "release",
            VersionType::OldBeta => "old_beta",
            VersionType::OldAlpha => "old_alpha",
        };
        write!(f, "{}", version_name)
    }
}
