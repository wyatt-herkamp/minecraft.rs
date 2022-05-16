use serde::Deserialize;
use std::collections::HashMap;
use crate::game_files::release::rule::Rule;

#[derive(Deserialize, Debug)]
pub struct Library {
    pub downloads: LibraryDownloads,
    pub name: String,
    pub rules: Option<Vec<Rule>>,
    pub extract: Option<LibraryExtract>,
    pub natives: Option<HashMap<String, String>>,
}
#[derive(Deserialize, Debug)]
pub struct Artifact {
    pub sha1: String,
    pub size: u32,
    pub url: String,
    pub path: String,
}

#[derive(Deserialize, Debug)]
pub struct LibraryDownloads {
    pub artifact: Option<Artifact>,
    #[serde(default)]
    pub classifiers: HashMap<String, Artifact>,
}

#[derive(Deserialize, Debug)]
pub struct LibraryExtract {
    pub exclude: Vec<String>,
}
