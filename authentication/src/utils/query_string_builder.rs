#![allow(dead_code)]
use std::{borrow::Cow, fmt::Display};

use reqwest::Body;
use url::Url;

/// Assumes all Input query string safe. This might change in the future
#[derive(Debug, Default)]
pub struct QueryString<'a> {
    query: Vec<KVP<'a>>,
    str_size: usize,
}
impl<'a> QueryString<'a> {
    pub fn new_with_capacity(capacity: usize) -> Self {
        QueryString {
            query: Vec::with_capacity(capacity),
            str_size: 0,
        }
    }
    pub fn add_str(mut self, key: &'static str, value: &'a str) -> Self {
        self.str_size += value.len() + key.len() + 1;
        self.query.push(KVP {
            key,
            value: value.into(),
        });
        self
    }
    pub fn add_optional_str(mut self, key: &'static str, value: Option<&'a str>) -> Self {
        if let Some(value) = value {
            self.str_size += value.len() + key.len() + 1;
            self.query.push(KVP {
                key,
                value: value.into(),
            });
        }
        self
    }
    pub fn add_string(mut self, key: &'static str, value: String) -> Self {
        self.str_size += value.len() + key.len() + 1;

        self.query.push(KVP {
            key,
            value: value.into(),
        });
        self
    }
    pub fn add_optional_string(mut self, key: &'static str, value: Option<String>) -> Self {
        if let Some(value) = value {
            self.str_size += value.len() + key.len() + 1;

            self.query.push(KVP {
                key,
                value: value.into(),
            });
        }
        self
    }

    pub fn add_display<T: Display>(mut self, key: &'static str, value: T) -> Self {
        let value = value.to_string();
        self.str_size += value.len() + key.len() + 1;

        self.query.push(KVP {
            key,
            value: value.into(),
        });
        self
    }
    pub fn add_optional_display<T: Display>(mut self, key: &'static str, value: Option<T>) -> Self {
        if let Some(value) = value {
            let value = value.to_string();
            self.str_size += value.len() + key.len() + 1;
            self.query.push(KVP {
                key,
                value: value.into(),
            });
        }
        self
    }

    pub fn build(self) -> String {
        let mut query = String::with_capacity(self.str_size + self.query.len());
        let last = self.query.len() - 1;
        for i in 0..self.query.len() {
            let kvp = &self.query[i];
            query.push_str(kvp.key);
            query.push('=');
            query.push_str(&kvp.value);
            if i != last {
                query.push('&')
            }
        }
        query
    }
    pub fn build_with_url(self, url: &str) -> Result<Url, url::ParseError> {
        Url::parse_with_params(url, self.query.iter().map(|kvp| (kvp.key, &kvp.value)))
    }
}

impl Into<Body> for QueryString<'_> {
    fn into(self) -> Body {
        let string = self.build();
        Body::from(string)
    }
}
#[derive(Debug, Hash, Eq, PartialEq)]
pub struct KVP<'a> {
    pub key: &'static str,
    pub value: Cow<'a, str>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_query_string() {
        let query = QueryString::default()
            .add_str("key", "value")
            .add_optional_string("key2", Some("value2".to_string()))
            .add_optional_display("key3", Some(true));
        assert_eq!(query.query.len(), 3);
        let query = query.build();
        assert!(query.contains("key=value"));
        assert!(query.contains("key2=value2"));
        assert!(query.contains("key3=true"));
        println!("{query:?}")
    }
}
