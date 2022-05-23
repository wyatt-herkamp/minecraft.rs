use serde::Deserialize;
use std::collections::HashMap;
use crate::game_files::release::data::ReleaseData;
use crate::game_files::release::rule::{Rule, RuleType};

impl ReleaseData {
    pub fn get_libraries_to_download(&self, os: String, arch: String, version: Option<String>) -> Vec<&Library> {
        let mut libraries = Vec::new();
        for library in &self.libraries.iter() {
            if let Some(rules) = library.rules.as_ref() {
                for rule in rules {
                    let mut should_download = false;
                    match rule.action {
                        RuleType::Allow => {
                            if rule.should_do_os(os.as_str(), arch.as_str(), version.clone()) {
                                should_download = true;
                            }
                        }
                        RuleType::Disallow => {
                            if rule.should_do_os(os.as_str(), arch.as_str(), version.clone()) {
                                should_download = false;
                            }
                        }
                    }
                    if should_download{
                        libraries.push(library)
                    }
                }
            } else {
                libraries.push(library)
            }
        }
        libraries
    }
}

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
