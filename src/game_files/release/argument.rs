use derive_more::From;
use serde::{Deserialize, Serialize};

use crate::game_files::release::rule::Rule;

/// The Arguments for the game
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Arguments {
    /// The arguments passed into the game
    pub game: Vec<Argument>,
    /// Arguments for the JVM
    pub jvm: Vec<Argument>,
}

/// Argument Type
#[derive(Debug, Clone, PartialEq, Eq, From, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Argument {
    RuledArgument(RuledArgument),
    Simple(String),
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuledArgument {
    /// The Value or what is added
    #[serde(with = "crate::utils::serde_utils::string_or_array")]
    value: Vec<String>,
    /// Rules required
    rules: Vec<Rule>,
}
impl<'a> From<&'a str> for Argument {
    fn from(value: &'a str) -> Self {
        Argument::Simple(value.to_owned())
    }
}
