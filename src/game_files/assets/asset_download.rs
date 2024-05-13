use std::path::PathBuf;

use tokio::fs::create_dir_all;

use crate::{
    game_files::assets::{content_hash, file_path, Asset},
    utils::download::{Download, DownloadToFile},
    Error,
};

/// A wrap around the Download type to allow you to specific just a asset directory. Instead of the entire type
#[derive(Debug)]
pub struct AssetDownload<'a, D> {
    pub(crate) asset: &'a Asset,
    pub(crate) download: D,
}

impl AssetDownload<'_, Download> {
    /// Downloads a file. If the file already exists. it will be overwritten
    /// `asset_dir` is the directory to download assets to
    pub async fn download(self, asset_dir: PathBuf, map_to_resources: bool) -> Result<(), Error> {
        self.download_with_subscriber(asset_dir, map_to_resources, |_| {})
            .await
    }
    /// Downloads a file. if a file already exists. it will be overwritten
    /// `subscriber` is a function that is called whenever a new set of bytes is downloaded and written. Param is the number of bytes download
    /// `asset_dir` is the directory to download assets to
    pub async fn download_with_subscriber<F>(
        self,
        asset_dir: PathBuf,
        map_to_resources: bool,
        subscriber: F,
    ) -> Result<(), Error>
    where
        F: Fn(usize),
    {
        if !asset_dir.exists() {
            create_dir_all(&asset_dir).await?;
        }
        let sub = content_hash(&self.asset.data.hash);
        let asset_file = asset_dir.join(file_path(
            self.asset.name.as_str(),
            (sub, &self.asset.data.hash),
            map_to_resources,
        ));
        self.download
            .download_with_subscriber(asset_file, subscriber)
            .await?;
        Ok(())
    }
}

impl AssetDownload<'_, DownloadToFile> {
    /// Downloads a file. If the file already exists. it will be overwritten
    /// `asset_dir` is the directory to download assets to
    pub async fn download(self, map_to_resources: bool) -> Result<(), Error> {
        self.download_with_subscriber(map_to_resources, |_| {})
            .await
    }
    /// Downloads a file. if a file already exists. it will be overwritten
    /// `subscriber` is a function that is called whenever a new set of bytes is downloaded and written. Param is the number of bytes download
    /// `asset_dir` is the directory to download assets to
    pub async fn download_with_subscriber<F>(
        self,
        map_to_resources: bool,
        subscriber: F,
    ) -> Result<(), Error>
    where
        F: Fn(usize),
    {
        let sub = content_hash(&self.asset.data.hash);
        let asset_file = self.download.location.join(file_path(
            self.asset.name.as_str(),
            (sub, &self.asset.data.hash),
            map_to_resources,
        ));
        self.download
            .download
            .download_with_subscriber(asset_file, subscriber)
            .await?;
        Ok(())
    }
}

impl<'a> Into<Download> for AssetDownload<'a, Download> {
    fn into(self) -> Download {
        self.download
    }
}
