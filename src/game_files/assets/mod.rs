
use std::path::PathBuf;
use substring::Substring;
use crate::{APIClient, Error};
use crate::game_files::assets::asset_download::AssetDownload;
use crate::game_files::assets::data::AssetResponse;
use crate::utils::download::Download;

pub mod asset_download;
pub mod data;

/// Release contains a reference to the APIClient and the internal data gotten from the asset
pub struct Asset<'a> {
    pub(crate) client: &'a APIClient,
    pub data: AssetResponse,
    pub name: String,
}

impl<'a> Asset<'a> {
    /// Creates an Asset Download for downloading assets
    pub async fn download(&self) -> Result<AssetDownload<'_, Download<'_>>, Error> {
        let (sub, hash) = content_hash(self.data.hash.clone());
        let url = self
            .client
            .game_files
            .create_resource_url(format!("{}/{}", sub, hash));
        Ok(AssetDownload {
            asset: self,
            download: Download {
                url,
                file_size: self.data.size as usize,
                client: self.client,
            },
        })
    }
}

/// Internal Use Only
/// Hashes the string in sha1
pub(crate) fn content_hash(hash: String) -> (String, String) {
    // Sub also known as NickAc
    let sub = hash.substring(0, 2).to_string();
    (sub, hash)
}

/// Creates the file path based on the File name and map_to_resources
/// `pre_hash` if the hash was created for the name. You can pass a reference
pub fn file_path<S: AsRef<str>>(name: S, pre_hash: (S, S), map_to_resources: bool) -> PathBuf {
    if map_to_resources {
        PathBuf::from(name.as_ref())
    } else {
        PathBuf::from("objects")
            .join(pre_hash.0.as_ref())
            .join(pre_hash.1.as_ref())
    }
}
