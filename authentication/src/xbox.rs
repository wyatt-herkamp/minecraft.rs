use crate::error::{BadResponseOrError, IntoResult, ResponseError};
use crate::utils::BetterResponseToJson;
use crate::{AuthenticationClient, InternalError, ACCEPT};
use chrono::{DateTime, Utc};
use reqwest::header::CONTENT_TYPE;

use reqwest::{Body, IntoUrl, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Debug;
use thiserror::Error;
use tracing::{debug, error};
#[derive(Debug, Clone, Error)]
#[error("{response}")]
pub struct XSTSError {
    pub status_code: StatusCode,
    pub response: XSTSErrorResponse,
}
/// This Type Wraps the XErr that XSTS can respond with
#[derive(Serialize, Deserialize, Error, Debug, Clone)]
#[error("XSTS Error: {error_code}")]
pub struct XSTSErrorResponse {
    #[serde(rename = "Identity")]
    pub identity: String,
    #[serde(rename = "Message")]
    pub message: String,
    #[serde(rename = "Redirect")]
    pub redirect: String,
    #[serde(rename = "XErr")]
    pub error_code: String,
}
impl ResponseError for XSTSError {
    fn status_code(&self) -> reqwest::StatusCode {
        self.status_code
    }

    async fn from_err(response: Response) -> Result<Self, InternalError> {
        let status_code = response.status();
        let response = response.better_to_json::<XSTSErrorResponse>().await;
        match response {
            Ok(ok) => Ok(XSTSError {
                response: ok,
                status_code,
            }),
            Err(err) => {
                error!(
                    ?err,
                    "Could not parse the bad response from Microsoft. So 10/10 Job Microsoft"
                );
                Err(err.into())
            }
        }
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
        self.display_claims
            .x_ui
            .get(0)
            .and_then(|value| Some(&value.user_hash))
    }
}

impl AuthenticationClient {
    /// Creates a Xbox Live Token from the Microsoft Authentication Token
    pub async fn authenticate_xbl<S: AsRef<str>>(
        &self,
        authorization_token: S,
    ) -> Result<XboxLiveResponse, BadResponseOrError<XSTSError>> {
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

        self.make_request(
            "https://user.auth.xboxlive.com/user/authenticate",
            serde_json::to_string(&value).unwrap(),
        )
        .await
    }

    /// Generates an Xbox Live security Token from the xbox_live token
    /// Use [authenticate_xbl](authenticate_xbl) to get the xbox_live_token
    pub async fn authenticate_xsts<S: AsRef<str>>(
        &self,
        xbox_live_token: S,
    ) -> Result<XboxLiveResponse, BadResponseOrError<XSTSError>> {
        let value = json!({
                "Properties": {
                 "SandboxId": "RETAIL",
                 "UserTokens": [xbox_live_token.as_ref()]
               },
               "RelyingParty": "rp://api.minecraftservices.com/",
               "TokenType": "JWT"
        });
        let value = serde_json::to_string(&value).unwrap();
        self.make_request("https://xsts.auth.xboxlive.com/xsts/authorize", value)
            .await
    }

    /// Internal Use to limit the amount of code I am repeating

    async fn make_request<D: DeserializeOwned>(
        &self,
        url: impl IntoUrl,
        content: impl Into<Body>,
    ) -> Result<D, BadResponseOrError<XSTSError>> {
        // Append Accept and CONTENT_TYPE Headers
        let request = self
            .http_client
            .post(url)
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .body(content)
            .build()
            .map_err(InternalError::from)?;
        debug!(?request, "Making Request to Microsoft Authentication");
        // Create Request
        let response = self
            .http_client
            .execute(request)
            .await
            .map_err(InternalError::from)?
            .into_result::<XSTSError>()
            .await??;
        let response = response.text().await.map_err(InternalError::from)?;
        debug!(?response);
        let parsed: D = serde_json::from_str(&response).map_err(InternalError::from)?;
        Ok(parsed)
    }
}
