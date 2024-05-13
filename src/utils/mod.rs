use crate::Error;
pub(crate) mod serde_utils;
use std::path::PathBuf;

use reqwest::{Client, Url};
use tokio::{
    fs::{create_dir_all, OpenOptions},
    io::AsyncWriteExt,
};

pub mod download;
pub async fn download_with_subscriber<F>(
    url: Url,
    reqwest: &Client,
    location: PathBuf,
    subscriber: F,
) -> Result<(), Error>
where
    F: Fn(usize),
{
    if let Some(parent) = location.parent() {
        if !parent.exists() {
            create_dir_all(&parent).await?;
        }
    }
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&location)
        .await?;
    let mut source = reqwest.get(url).send().await?;
    while let Some(chunk) = source.chunk().await.unwrap() {
        file.write_all(&chunk).await?;
        subscriber(chunk.len());
    }
    Ok(())
}
