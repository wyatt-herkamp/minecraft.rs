use crate::game_files::release::rule::Rule;
use serde::de::{MapAccess, Visitor};
use serde::{de, Deserialize, Deserializer};
use serde_json::Value;
use std::convert::Infallible;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

/// The Arguments for the game
#[derive(Deserialize, Debug)]
pub struct Arguments {
    /// The arguments passed into the game
    pub game: Vec<Argument>,
    /// Arguments for the JVM
    pub jvm: Vec<Argument>,
}

/// Argument Type
#[derive(Debug)]
pub struct Argument {
    /// The Value or what is added
    pub value: Vec<String>,
    /// Rules required
    pub rules: Vec<Rule>,
}

impl FromStr for Argument {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Argument {
            value: vec![s.to_string()],
            rules: Default::default(),
        })
    }
}

impl<'de> Deserialize<'de> for Argument {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ArgumentVisitor;
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Value,
            Rules,
        }

        impl<'de> Visitor<'de> for ArgumentVisitor {
            type Value = Argument;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("String or Struct as Argument")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Argument::from_str(v).unwrap())
            }
            fn visit_map<V>(self, mut map: V) -> Result<Argument, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut value = None;
                let mut rules = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Value => {
                            if value.is_some() {
                                return Err(de::Error::duplicate_field("value"));
                            }

                            value = Some(map.next_value()?);
                        }
                        Field::Rules => {
                            if rules.is_some() {
                                return Err(de::Error::duplicate_field("rules"));
                            }

                            rules = Some(map.next_value()?);
                        }
                    }
                }
                let value_ob: Value = value.ok_or_else(|| de::Error::missing_field("value"))?;
                let value = if value_ob.is_array() {
                    serde_json::from_value(value_ob).unwrap()
                } else {
                    vec![serde_json::from_value(value_ob).unwrap()]
                };
                Ok(Argument {
                    value,
                    rules: rules.unwrap_or_default(),
                })
            }
        }

        deserializer.deserialize_any(ArgumentVisitor {})
    }
}
