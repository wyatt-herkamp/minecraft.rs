use std::fmt::{Debug, Formatter};
use chrono::{DateTime, Utc};
use reqwest::{Body, StatusCode};
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::json;
use crate::{ACCEPT, APIClient, Error};
/// This Type Wraps the XErr that XSTS can respond with
#[derive(Serialize, Deserialize)]
pub struct XSTSError {
    #[serde(rename = "Identity")]
    pub identity: String,
    #[serde(rename = "Message")]
    pub message: String,
    #[serde(rename = "Redirect")]
    pub redirect: String,
    #[serde(rename = "XErr")]
    pub error_code: String,
}

impl From<String> for XSTSError {
    fn from(value: String) -> Self {
        serde_json::from_str(&value).unwrap()
    }
}

impl Debug for XSTSError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

#[derive(Deserialize, Debug)]
pub struct XboxDisplayClaims {
    #[serde(rename = "xui")]
    pub x_ui: Vec<XUI>,
}


#[derive(Deserialize, Debug)]
pub struct XUI {
    /// Should be saved
    #[serde(rename = "uhs")]
    pub user_hash: String,
}

/// Standard Response from Xbox Live
#[derive(Deserialize, Debug)]
pub struct XboxLiveResponse {
    #[serde(rename = "IssueInstant")]
    pub issue_instance: DateTime<Utc>,
    #[serde(rename = "NotAfter")]
    pub not_after: DateTime<Utc>,
    /// Should be saved
    #[serde(rename = "Token")]
    pub token: String,
    #[serde(rename = "DisplayClaims")]
    pub display_claims: XboxDisplayClaims,
}

impl XboxLiveResponse {
    /// Gets the User Hash
    /// # Panics
    /// If x_ui didn't provide a user hash
    pub fn get_user_hash_unsafe(&self) -> &String {
        &self.display_claims.x_ui.get(0).unwrap().user_hash
    }
    pub fn get_user_hash(&self) -> Option<&String> {
        self.display_claims.x_ui.get(0).and_then(|value| Some(&value.user_hash))
    }
}


impl APIClient{
    /// Creates a Xbox Live Token from the Microsoft Authentication Token
    pub async fn authenticate_xbl<S: AsRef<str>>(
        &self,
        authorization_token: S,
    ) -> Result<Result<XboxLiveResponse, XSTSError>, Error> {
        let authorization_url =
            format!("https://user.auth.xboxlive.com/user/authenticate");
        let rps = format!("d={}", authorization_token.as_ref());
        let value = json!({
                "Properties": {
                  "AuthMethod": "RPS",
                  "SiteName": "user.auth.xboxlive.com",
                  "RpsTicket": rps
               },
               "RelyingParty": "http://auth.xboxlive.com",
               "TokenType": "JWT"
        });


        self.make_request("https://xsts.auth.xboxlive.com/xsts/authorize", serde_json::to_string(&value).unwrap()).await
    }

    /// Generates an Xbox Live security Token from the xbox_live token
    /// Use [authenticate_xbl](authenticate_xbl) to get the xbox_live_token
    pub async fn authenticate_xsts<S: AsRef<str>>(
        &self,
        xbox_live_token: S,
    ) -> Result<Result<XboxLiveResponse, XSTSError>, Error> {
        let authorization_url = "https://xsts.auth.xboxlive.com/xsts/authorize";
        let value = json!({
                "Properties": {
                 "SandboxId": "RETAIL",
                 "UserTokens": [xbox_live_token.as_ref()]
               },
               "RelyingParty": "rp://api.minecraftservices.com/",
               "TokenType": "JWT"
        });
        let value = serde_json::to_string(&value).unwrap();
        self.make_request("https://xsts.auth.xboxlive.com/xsts/authorize", value).await
    }

    /// Internal Use to limit the amount of code I am repeating
    async fn make_request<D: DeserializeOwned, E: From<String>>(&self, url: &str, content: String) -> Result<Result<D, E>, Error> {
        match self.process_json::<D>(self
            .http_client
            .post(url)
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .body(content))
            .await {
            Ok(ok) => {
                Ok(Ok(ok))
            }
            Err(error) => {
                match error {
                    Error::BadResponse(response) => {
                        let response = response.text().await?;
                        Ok(Err(E::from(response)))
                    }
                    error => {
                        Err(error)
                    }
                }
            }
        }
    }
}