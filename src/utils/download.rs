use std::{
    fmt::{Debug, Formatter},
    path::PathBuf,
};

use reqwest::{Response, Url};

use crate::{APIClient, Error};

/// A generic Download handler and type
/// Contains a Response that we wrap to to make file downloading easy
#[derive(Clone)]
pub struct Download {
    /// URL to the download
    pub(crate) url: Url,
    /// The number of bytes the download is
    pub file_size: usize,
    /// A Reference to the API Client
    pub(crate) client: APIClient,
}

impl Download {
    /// Downloads a file. If the file already exists. it will be overwritten
    pub async fn download(self, location: PathBuf) -> Result<(), Error> {
        self.download_with_subscriber(location, |_| {}).await
    }
    /// Downloads a file. if a file already exists. it will be overwritten
    /// `subscriber` is a function that is called whenever a new set of bytes is downloaded and written. Param is the number of bytes download
    pub async fn download_with_subscriber<F>(
        self,
        location: PathBuf,
        subscriber: F,
    ) -> Result<(), Error>
    where
        F: Fn(usize),
    {
        super::download_with_subscriber(self.url, &self.client.http_client, location, subscriber)
            .await
    }
    /// Returns the bytes for the download
    pub async fn get_bytes(self) -> Result<Vec<u8>, Error> {
        Ok(self.to_request().await?.bytes().await?.to_vec())
    }
    /// Turns the Download into a request. Allowing full control of the download
    pub async fn to_request(self) -> Result<Response, Error> {
        self.client
            .http_client
            .get(self.url)
            .send()
            .await
            .map_err(Error::from)
    }
}

impl Debug for Download {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "URL {} with size of {}",
            self.url.as_str(),
            self.file_size
        )
    }
}

/// This is a [Download](Download) with a [PathBuf](std::path::PathBuf)
#[derive(Clone)]
pub struct DownloadToFile {
    pub(crate) location: PathBuf,
    pub(crate) download: Download,
}
impl DownloadToFile {
    pub(crate) fn new(download: Download, location: PathBuf) -> DownloadToFile {
        DownloadToFile { location, download }
    }
    /// Downloads a file. If the file already exists. it will be overwritten
    pub async fn download(self) -> Result<(), Error> {
        self.download_with_subscriber(|_| {}).await
    }
    /// Downloads a file. if a file already exists. it will be overwritten
    /// `subscriber` is a function that is called whenever a new set of bytes is downloaded and written. Param is the number of bytes download
    pub async fn download_with_subscriber<F>(self, subscriber: F) -> Result<(), Error>
    where
        F: Fn(usize),
    {
        super::download_with_subscriber(
            self.download.url,
            &self.download.client.http_client,
            self.location,
            subscriber,
        )
        .await
    }
}

impl Debug for DownloadToFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} to location {:?}", self.download, self.location)
    }
}
