use serde::de::{MapAccess, Visitor};
use serde::{de, Deserialize, Deserializer};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Write;

/// The Rule Type
#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum RuleType {
    /// States the action will be preformed if the Rule requirements are met
    Allow,
    /// States the action will not be preformed if the Rule requirements are met
    Disallow,
}

/// Rule Requirements
#[derive(Debug)]
pub enum RuleRequirement {
    /// OS Requirements
    OS(Vec<RuleOS>),
    /// Features enabled
    Features(HashMap<String, bool>),
}

/// The OS Requirement
#[derive(Debug)]
pub enum RuleOS {
    /// OS Name
    Name(String),
    /// OS Arch
    Arch(String),
    /// OS Version
    Version(String),
    /// A Catch All
    Other { key: String, value: String },
}

/// Sets the rules for the [Argument](Argument) or [Library](Library)
/// Custom Deserialization done
#[derive(Debug)]
pub struct Rule {
    /// What is to happen on if the requirements are met
    action: RuleType,
    /// The Rule Requirements
    /// As of now(05/12/2022) only one RuleRequirement exists per option. However, I am prepping for the worst
    requirements: Vec<RuleRequirement>,
}

impl<'de> Deserialize<'de> for Rule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RuleVisitor;
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Action,
            Os,
            Features,
        }

        impl<'de> Visitor<'de> for RuleVisitor {
            type Value = Rule;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Expecting Rule Type")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Rule, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut action: Option<RuleType> = None;
                let mut requirements: Vec<RuleRequirement> = Vec::new();
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Action => {
                            if action.is_some() {
                                return Err(de::Error::duplicate_field("action"));
                            }

                            action = Some(map.next_value()?);
                        }

                        Field::Os => {
                            let value: HashMap<String, String> = map.next_value()?;
                            let mut os = Vec::new();
                            for (key, value) in value {
                                match key.as_str() {
                                    "name" => os.push(RuleOS::Name(value)),
                                    "version" => os.push(RuleOS::Version(value)),
                                    "arch" => os.push(RuleOS::Arch(value)),
                                    _ => os.push(RuleOS::Other { key, value }),
                                }
                            }
                            let requirement = RuleRequirement::OS(os);
                            requirements.push(requirement)
                        }
                        Field::Features => {
                            let requirement = RuleRequirement::Features(map.next_value()?);
                            requirements.push(requirement)
                        }
                    }
                }
                let action = action.ok_or_else(|| de::Error::missing_field("action"))?;
                Ok(Rule {
                    action,
                    requirements,
                })
            }
        }

        deserializer.deserialize_any(RuleVisitor {})
    }
}
