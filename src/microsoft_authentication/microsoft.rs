use crate::utils::query_string_builder::QueryString;
use crate::{APIClientWithAuth, AuthProperties, Error};
use reqwest::header::CONTENT_TYPE;

use reqwest::Body;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use url::Url;

use thiserror::Error;

/// The Response from a Authorization Request for Microsoft
/// Documented [Here](https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-auth-code-flow#successful-response-2)
#[derive(Deserialize, Debug)]
pub struct AuthorizationTokenResponse {
    pub token_type: String,
    pub scope: String,
    /// You care about this value
    /// This is used for getting a Xbox Live Token
    pub access_token: String,
    /// Also this value
    /// This is used for getting a new access token
    pub refresh_token: String,
    pub user_id: String,
    pub expires_in: i64,
}

/// An Error Response from Microsoft
/// [Microsoft Error Response](https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-auth-code-flow#error-response-1)
#[derive(Serialize, Deserialize, Error, Debug)]
#[error("Microsoft Error Code Error: {error}")]
pub struct MicrosoftError {
    pub error_codes: Vec<i64>,
    pub error: String,
    pub error_description: String,
    pub timestamp: String,
    pub trace_id: String,
    pub correlation_id: String,
}

impl From<String> for MicrosoftError {
    fn from(value: String) -> Self {
        serde_json::from_str(&value).unwrap()
    }
}

pub const MICROSOFT_LOGIN_URL: &str = "https://login.microsoftonline.com/consumer";
pub const MICROSOFT_SCOPE: &str = "XboxLive.signin%20offline_access";

pub trait MicrosoftAuthorizationType {}

/// Exists when doing refresh_tokens
pub struct RefreshToken;

impl RefreshToken {
    /// Creates a new Grant Code Authentication
    pub fn new(api_client: &APIClientWithAuth) -> MicrosoftAuthorization<'_, RefreshToken> {
        MicrosoftAuthorization {
            client: api_client,
            internal: RefreshToken,
        }
    }
}

impl MicrosoftAuthorizationType for RefreshToken {}

/// Grant Code follows the Microsoft Authentication Schema [Here](https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-auth-code-flow)
/// Requires the redirect url to be set
pub struct GrantCode {
    pub redirect_url: String,
}

impl GrantCode {
    /// Creates a new Grant Code Authentication
    pub fn new<S: Into<String>>(
        redirect_url: S,
        api_client: &APIClientWithAuth,
    ) -> MicrosoftAuthorization<'_, GrantCode> {
        MicrosoftAuthorization {
            client: api_client,
            internal: GrantCode {
                redirect_url: redirect_url.into(),
            },
        }
    }
}

impl MicrosoftAuthorizationType for GrantCode {}

/// Device Code follows the Microsoft Authentication Schema [Here](https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-device-code)
/// This will not require a redirect URL for authentication
pub struct DeviceCode;

impl MicrosoftAuthorizationType for DeviceCode {}

pub struct MicrosoftAuthorization<'a, T: MicrosoftAuthorizationType> {
    pub client: &'a APIClientWithAuth,
    pub internal: T,
}

impl<'a, T: MicrosoftAuthorizationType> MicrosoftAuthorization<'a, T> {
    /// Refreshes the Authorization Token via the Refresh Token
    /// Refresh Token is given in the [AuthorizationTokenResponse](AuthorizationTokenResponse)
    /// # Error
    /// Error responds in two types. The value is network or parsing errors. The inner value is a Microsoft Error Response
    pub async fn refresh_token<S: AsRef<str>>(
        &self,
        refresh_token: S,
    ) -> Result<Result<AuthorizationTokenResponse, MicrosoftError>, Error> {
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
    async fn make_request<D: DeserializeOwned, E: From<String>>(
        &self,
        url: String,
        content: impl Into<Body>,
    ) -> Result<Result<D, E>, Error> {
        match self
            .client
            .process_json::<D>(
                self.client
                    .http_client
                    .post(url)
                    .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(content),
            )
            .await
        {
            Ok(ok) => Ok(Ok(ok)),
            Err(error) => match error {
                Error::BadResponse(response) => {
                    let response = response.text().await?;
                    Ok(Err(E::from(response)))
                }
                error => Err(error),
            },
        }
    }
}

impl<'a> MicrosoftAuthorization<'a, GrantCode> {
    /// Have a user go to this URL to generate the access_code
    /// Documented [Here](https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-auth-code-flow#request-an-authorization-code)
    ///
    /// [Successful Response](https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-auth-code-flow#successful-response)
    /// [Error Response](https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-auth-code-flow#error-codes-for-authorization-endpoint-errors)
    pub fn generate_login_url(&self) -> Result<Url, Error> {
        let AuthProperties {
            azura_microsoft_client,
            ..
        } = &self.client.auth_properties;
        let query_string = QueryString::new_with_capacity(4)
            .add_str("client_id", azura_microsoft_client)
            .add_str("response_type", "code")
            .add_str("redirect_uri", &self.internal.redirect_url)
            .add_str("scope", MICROSOFT_SCOPE);
        query_string
            .build_with_url(&format!("{MICROSOFT_LOGIN_URL}/oauth2/v2.0/authorize"))
            .map_err(Error::URLParse)
    }
    /// Generates a Authorization Token from the access code
    /// The access code should not be saved. This should immediately be given to this function
    /// Documented Here [Here](https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-auth-code-flow#request-an-access-token-with-a-client_secret)
    /// # Error
    /// Error responds in two types. The value is network or parsing errors. The inner value is a Microsoft Error Response
    pub async fn get_authorization_token(
        &self,
        access_code: String,
    ) -> Result<Result<AuthorizationTokenResponse, MicrosoftError>, Error> {
        let AuthProperties {
            azura_microsoft_client,
            ..
        } = &self.client.auth_properties;
        let authorization_url = format!("{MICROSOFT_LOGIN_URL}/oauth2/v2.0/token");
        let query_string = QueryString::new_with_capacity(5)
            .add_str("client_id", azura_microsoft_client)
            .add_str("code", &access_code)
            .add_str("scope", MICROSOFT_SCOPE)
            .add_str("grant_type", "authorization_code")
            .add_str("redirect_uri", &self.internal.redirect_url);
        self.make_request(authorization_url, query_string).await
    }
}
