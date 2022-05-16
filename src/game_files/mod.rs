pub mod version_manifest;
pub mod version_type;
pub mod release;
pub mod assets;

use reqwest::Url;
use crate::APIClient;


#[derive(Clone, Debug)]
pub struct GameFilesAPIBuilder {
    pub resource_base: String,
    pub library_base: String,
    pub launcher_meta: String,
}

impl Default for GameFilesAPIBuilder {
    fn default() -> Self {
        GameFilesAPIBuilder {
            resource_base: "https://resources.download.minecraft.net".to_string(),
            library_base: "https://libraries.minecraft.net".to_string(),
            launcher_meta: "https://launchermeta.mojang.com".to_string(),
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

