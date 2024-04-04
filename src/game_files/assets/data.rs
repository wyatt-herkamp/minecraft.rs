use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::game_files::assets::{content_hash, file_path, Asset};
use crate::utils::download::{Download, DownloadToFile};
use crate::{APIClient, Error};
use tokio::fs::create_dir_all;

/// The Asset File
/// Asset File Example 1.19 ['{launcher_meta}/v1/packages/c76d769e6bf9c90a7ffff1481a05563777356749/1.19.json'](https://launchermeta.mojang.com/v1/packages/c76d769e6bf9c90a7ffff1481a05563777356749/1.19.json)
#[derive(Deserialize)]
pub struct AssetFile {
    /// Will be found true on legacy versions.
    #[serde(default)]
    pub map_to_resources: bool,
    pub objects: HashMap<String, AssetResponse>,
}

/// Object found in the Map of the [AssetFile](AssetFile)
#[derive(Deserialize, Clone)]
pub struct AssetResponse {
    pub hash: String,
    pub size: u32,
}

impl AssetFile {
    /// Get an Asset from the the AssetFile
    pub fn get_asset<'a, S: AsRef<str>>(
        &self,
        client: &'a APIClient,
        name: S,
    ) -> Option<Asset<'a>> {
        self.objects.get(name.as_ref()).map(|value| Asset {
            client,
            data: value.clone(),
            name: name.as_ref().to_string(),
        })
    }
    /// Downloads a file. If the file already exists. it will be overwritten
    /// `asset_dir` is the directory to download assets to
    /// Returns a HashMap<String, [DownloadFile](crate::utils::download::DownloadFile<'a>)> Key is the hash of the file
    pub async fn download(
        self,
        client: &APIClient,
        asset_dir: PathBuf,
    ) -> Result<HashMap<String, DownloadToFile<'_>>, Error> {
        let mut downloads = HashMap::new();
        if !asset_dir.exists() {
            create_dir_all(&asset_dir).await?;
        }

        for (name, response) in self.objects {
            if downloads.contains_key(&response.hash) {
                continue;
            }
            let sub = content_hash(&response.hash);

            let asset_file = asset_dir.join(file_path(
                name.as_str(),
                (sub, &response.hash),
                self.map_to_resources,
            ));
            if asset_file.exists() {
                // Skip the files that already exist
                continue;
            }
            let url = client
                .game_files
                .create_resource_url(format!("{}/{}", &sub, &response.hash));

            downloads.insert(
                response.hash,
                DownloadToFile::new(
                    Download {
                        url,
                        file_size: response.size as usize,
                        client,
                    },
                    asset_file,
                ),
            );
        }
        Ok(downloads)
    }
}
