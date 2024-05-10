use crate::error::{BadResponseOrError, IntoResult, ResponseError};
use crate::utils::query_string_builder::QueryString;
use crate::{AuthProperties, AuthenticationClient, InternalError};
use reqwest::header::{ACCEPT, CONTENT_TYPE};
pub mod device;
use reqwest::{Body, IntoUrl, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::ops::Deref;
use tracing::{debug, error};
pub mod grant;

use thiserror::Error;

/// The Response from a Authorization Request for Microsoft
/// Documented [Here](https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-auth-code-flow#successful-response-2)
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct AuthorizationTokenResponse {
    pub token_type: String,
    pub scope: String,
    /// You care about this value
    /// This is used for getting a Xbox Live Token
    pub access_token: String,
    /// Also this value
    /// This is used for getting a new access token
    pub refresh_token: String,
    // I think this can be removed? pub user_id: String,
    pub expires_in: i64,
}

/// An Error Response from Microsoft
/// [Microsoft Error Response](https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-auth-code-flow#error-response-1)
#[derive(Error, Debug)]
#[error("Microsoft Error {inner} with status_code {status_code}")]
pub struct MicrosoftError {
    pub inner: InnerMicrosoftError,
    pub status_code: StatusCode,
}
impl Deref for MicrosoftError {
    type Target = InnerMicrosoftError;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Error)]
#[error("Error From Microsoft {error}")]
pub struct InnerMicrosoftError {
    pub error_codes: Vec<i64>,
    pub error: String,
    pub error_description: String,
    pub timestamp: String,
    pub trace_id: String,
    pub correlation_id: String,
}
impl ResponseError for MicrosoftError {
    fn status_code(&self) -> reqwest::StatusCode {
        self.status_code
    }

    async fn from_err(response: Response) -> Result<Self, InternalError> {
        let status_code = response.status();
        match response
            .json::<InnerMicrosoftError>()
            .await
            .map(|inner| MicrosoftError { inner, status_code })
        {
            Ok(ok) => Ok(ok),
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

pub const MICROSOFT_LOGIN_URL: &str = "https://login.microsoftonline.com/consumers";
pub const MICROSOFT_SCOPE: &str = "XboxLive.signin%20offline_access";

pub trait MicrosoftAuthorizationType {}

/// Exists when doing refresh_tokens
#[derive(Debug, Clone, Copy)]
pub struct RefreshToken;

impl RefreshToken {
    /// Creates a new Grant Code Authentication
    pub fn new(api_client: AuthenticationClient) -> MicrosoftAuthorization<RefreshToken> {
        MicrosoftAuthorization {
            client: api_client,
            internal: RefreshToken,
        }
    }
}

impl MicrosoftAuthorizationType for RefreshToken {}
#[derive(Debug, Clone)]
pub struct MicrosoftAuthorization<T: MicrosoftAuthorizationType> {
    pub client: AuthenticationClient,
    pub internal: T,
}

impl<T: MicrosoftAuthorizationType> MicrosoftAuthorization<T> {
    /// Refreshes the Authorization Token via the Refresh Token
    /// Refresh Token is given in the [AuthorizationTokenResponse](AuthorizationTokenResponse)
    /// # Error
    /// Error responds in two types. The value is network or parsing errors. The inner value is a Microsoft Error Response
    pub async fn refresh_token<S: AsRef<str>>(
        &self,
        refresh_token: S,
    ) -> Result<AuthorizationTokenResponse, BadResponseOrError<MicrosoftError>> {
        let AuthProperties {
            azura_microsoft_client,
            ..
        } = &self.client.auth_properties;
        let authorization_url = format!("{MICROSOFT_LOGIN_URL}/oauth2/v2.0/token");
        let refresh_token = refresh_token.as_ref();
        let query_string = QueryString::default()
            .add_str("client_id", azura_microsoft_client)
            .add_str("scope", MICROSOFT_SCOPE)
            .add_str("refresh_token", refresh_token)
            .add_str("grant_type", "refresh_token");
        self.make_request(authorization_url, query_string).await
    }
    /// Internal Use to limit the amount of code I am repeating
    async fn make_request<D: DeserializeOwned>(
        &self,
        url: impl IntoUrl,
        content: impl Into<Body>,
    ) -> Result<D, BadResponseOrError<MicrosoftError>> {
        // Append Accept and CONTENT_TYPE Headers
        let request = self
            .client
            .http_client
            .post(url)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(content)
            .header(ACCEPT, "application/json")
            .build()
            .map_err(InternalError::from)?;
        debug!(?request, "Making Request to Microsoft Authentication");
        // Create Request
        let response = self
            .client
            .http_client
            .execute(request)
            .await
            .map_err(InternalError::from)?
            .into_result::<MicrosoftError>()
            .await??;
        let response = response.text().await.map_err(InternalError::from)?;
        debug!(?response);
        let parsed: D = serde_json::from_str(&response).map_err(InternalError::from)?;
        Ok(parsed)
    }
}
