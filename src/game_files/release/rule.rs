use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::warn;

/// The Rule Type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RuleType {
    /// States the action will be preformed if the Rule requirements are met
    Allow,
    /// States the action will not be preformed if the Rule requirements are met
    Disallow,
}

/// Rule Requirements
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleRequirement {
    /// OS Requirements
    OS(Vec<RuleOS>),
    /// Features enabled
    Features(HashMap<String, bool>),
    Default,
}

/// The OS Requirement
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    /// What is to happen on if the requirements are met
    pub action: RuleType,
    /// The Rule Requirements
    pub requirement: RuleRequirement,
}

impl Rule {
    pub fn should_do(
        &self,
        os: &str,
        arch: &str,
        version: Option<String>,
        features_enabled: Vec<String>,
    ) -> bool {
        self.should_do_os(os, arch, version) && self.should_do_feature(features_enabled)
    }
    pub fn should_do_os(&self, os: &str, arch: &str, version: Option<String>) -> bool {
        if let RuleRequirement::OS(os_rules) = &self.requirement {
            let mut os_name_match = false;
            let mut os_arch_match = false;
            let mut os_version_match = true;
            for os_rule in os_rules.iter() {
                match &os_rule {
                    RuleOS::Name(name) => os_name_match = name.eq(os),
                    RuleOS::Arch(value) => os_arch_match = arch.eq(value),
                    RuleOS::Version(value) => {
                        if let Some(requirement) = version.as_ref() {
                            warn!("Version parsing from the manifest is not ready yet");
                            os_version_match = requirement.eq(value)
                        }
                    }
                    RuleOS::Other { .. } => {
                        continue;
                    }
                };
            }
            return os_arch_match && os_version_match && os_name_match;
        }

        true
    }

    pub fn should_do_feature(&self, features_enabled: Vec<String>) -> bool {
        if let RuleRequirement::Features(features) = &self.requirement {
            for (key, _) in features {
                if !features_enabled.contains(key) {
                    return false;
                }
            }
        }
        true
    }
}
mod _serde {
    use std::collections::HashMap;

    use serde::{
        de,
        de::{MapAccess, Visitor},
        ser::{Serialize, SerializeMap, Serializer},
        Deserialize, Deserializer,
    };

    use super::{Rule, RuleOS, RuleRequirement};
    use crate::game_files::release::rule::RuleType;
    impl Serialize for Rule {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut ser_struct = serializer.serialize_map(Some(2))?;
            ser_struct.serialize_entry("action", &self.action)?;
            match &self.requirement {
                RuleRequirement::OS(os) => {
                    let mut map = HashMap::<&str, &str>::new();
                    for rule in os {
                        match rule {
                            RuleOS::Name(name) => map.insert("name", name),
                            RuleOS::Arch(arch) => map.insert("arch", arch),
                            RuleOS::Version(version) => map.insert("version", version),
                            RuleOS::Other { key, value } => map.insert(key, value),
                        };
                    }
                    ser_struct.serialize_entry("os", &map)?;
                }
                RuleRequirement::Features(features) => {
                    ser_struct.serialize_entry("features", &features)?;
                }
                _ => {}
            }

            return ser_struct.end();
        }
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

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("Expecting Rule Type")
                }

                fn visit_map<V>(self, mut map: V) -> Result<Rule, V::Error>
                where
                    V: MapAccess<'de>,
                {
                    let mut action: Option<RuleType> = None;
                    let mut requirement: Option<RuleRequirement> = None;
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
                                let r = RuleRequirement::OS(os);
                                requirement = Some(r)
                            }
                            Field::Features => {
                                let r = RuleRequirement::Features(map.next_value()?);
                                requirement = Some(r)
                            }
                        }
                    }
                    let action = action.ok_or_else(|| de::Error::missing_field("action"))?;
                    let requirement = requirement.unwrap_or(RuleRequirement::Default);
                    Ok(Rule {
                        action,
                        requirement,
                    })
                }
            }

            deserializer.deserialize_any(RuleVisitor {})
        }
    }
}
