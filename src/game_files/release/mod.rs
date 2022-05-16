
use crate::{APIClient, Error};
use reqwest::Url;
use crate::game_files::assets::data::AssetFile;
use crate::game_files::release::data::ReleaseData;

pub mod argument;
pub mod data;
pub mod library;
pub mod rule;

/// Release contains a reference to the APIClient and the internal data gotten from the Release Data
pub struct Release<'a> {
    pub(crate) client: &'a APIClient,
    pub data: ReleaseData,
}

impl Release<'_> {
    /// Gets the Asset File from the Release
    pub async fn get_asset_file(&self) -> Result<AssetFile, Error> {
        let url = Url::parse(&self.data.asset_index.url).unwrap();
        self.client.process_json::<AssetFile>(self.client.http_client.get(url)).await
    }
}

impl Into<ReleaseData> for Release<'_> {
    fn into(self) -> ReleaseData {
        self.data
    }
}

impl AsRef<ReleaseData> for Release<'_> {
    fn as_ref(&self) -> &ReleaseData {
        &self.data
    }
}
