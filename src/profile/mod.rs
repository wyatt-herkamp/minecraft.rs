use reqwest::header::{AUTHORIZATION, HeaderName, HeaderValue};
use reqwest::{IntoUrl, RequestBuilder};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use crate::{APIClient, Error};



impl APIClient {
    pub fn profile(&self, token: &str) -> Profile<'_> {
        Profile {
            authorization: format!("Bearer {}", token),
            api: self,
        }
    }
}

pub struct Profile<'a> {
    authorization: String,
    api: &'a APIClient,
}


impl<'a> Profile<'a> {
    pub(crate) fn get<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.api.http_client.get(url).header(AUTHORIZATION, &self.authorization)
    }
    pub(crate) fn post_json<S: Serialize + ?Sized, U: IntoUrl>(&self, url: U, s: &S) -> RequestBuilder {
        self.api.http_client.get(url).header(AUTHORIZATION, &self.authorization).json(s)
    }
}

#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[derive(Deserialize, Debug)]
pub struct ProfileSkin {
    pub id: String,
    pub state: String,
    pub url: String,
    pub variant: String,
}

#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[derive(Deserialize, Debug)]
pub struct ProfileResponse {
    pub id: Uuid,
    pub name: String,
    pub skins: Vec<ProfileSkin>,
    // TODO parse this data
    pub capes: Vec<Value>,
}

impl<'a> Profile<'a> {
    /// Returns the User Profile for the authenticated User
    pub async fn get_profile(&self) -> Result<ProfileResponse, Error> {
        self.api.process_json(self.get("https://api.minecraftservices.com/minecraft/profile")).await
    }
}