use crate::game_files::assets::asset_download::AssetDownload;
use crate::game_files::assets::data::AssetResponse;
use crate::utils::download::Download;
use crate::{APIClient, Error};
use std::path::PathBuf;

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
        let sub = content_hash(&self.data.hash);
        let url = self
            .client
            .game_files
            .create_resource_url(format!("{}/{}", sub, &self.data.hash));
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
pub(crate) fn content_hash(hash: &str) -> &str {
    if hash.len() < 2 {
        return "";
    }
    let mut indices = hash.char_indices();

    let obtain_index = |(index, _char)| index;
    let str_len = hash.len();
    let start = indices.nth(0).map_or(str_len, &obtain_index);
    let end = indices.nth(1).map_or(str_len, &obtain_index);
    unsafe { hash.get_unchecked(start..end) }
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
#[cfg(test)]
mod tests {
    use crate::game_files::assets::content_hash;
    #[test]
    pub fn sub_string() {
        assert_eq!(content_hash("MY_HASH"), "MY");
        assert_eq!(content_hash("MY"), "MY");
        assert_eq!(content_hash(""), "");
    }
}
