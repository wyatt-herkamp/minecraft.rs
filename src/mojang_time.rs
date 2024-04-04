use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serializer};

const FORMAT: &str = "%Y-%m-%dT%H:%M:%S%:z";
#[allow(dead_code)]
pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = format!("{}", date.format(FORMAT));
    serializer.serialize_str(&s)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    DateTime::parse_from_str(&s, FORMAT)
        .map_err(serde::de::Error::custom)
        .map(|date| date.to_utc())
}
