pub mod assets;
pub mod release;
pub mod version_manifest;
pub mod version_type;

use std::borrow::Cow;

use crate::APIClient;
use reqwest::Url;
use serde::{Deserialize, Serialize};
pub static RESOURCE_URL_BASE: &str = "https://resources.download.minecraft.net";
pub static LIBRARY_URL_BASE: &str = "https://libraries.minecraft.net";
pub static LAUNCHER_META_URL_BASE: &str = "https://launchermeta.mojang.com";

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
