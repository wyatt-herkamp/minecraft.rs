use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::str::FromStr;
use reqwest::Body;
use reqwest::header::CONTENT_TYPE;
use serde::de::DeserializeOwned;
use crate::{APIClient, Error};
use serde::{Serialize, Deserialize};



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
#[derive(Serialize, Deserialize)]
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

impl Debug for MicrosoftError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

pub const MICROSOFT_LOGIN_URL: &str = "https://login.microsoftonline.com/consumer";
pub const MICROSOFT_SCOPE: &str = "XboxLive.signin%20offline_access";

pub trait MicrosoftAuthorizationType {}

/// Exists when doing refresh_tokens
pub struct RefreshToken;

impl RefreshToken {
    /// Creates a new Grant Code Authentication
    pub fn new(api_client: &APIClient) -> MicrosoftAuthorization<'_, RefreshToken> {
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
    pub fn new<S: Into<String>>(redirect_url: S, api_client: &APIClient) -> MicrosoftAuthorization<'_, GrantCode> {
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
    pub client: &'a APIClient,
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
        let authorization_url =
            format!("{MICROSOFT_LOGIN_URL}/oauth2/v2.0/token");
        let refresh_token = refresh_token.as_ref();
        let content = format!("client_id={client_id}&scope={MICROSOFT_SCOPE}&refresh_token={refresh_token}&grant_type=refresh_token&", client_id = self.client.auth_properties.azura_microsoft_client);
        self.make_request(authorization_url, content).await
    }
    /// Internal Use to limit the amount of code I am repeating
    async fn make_request<D: DeserializeOwned, E: From<String>>(&self, url: String, content: String) -> Result<Result<D, E>, Error> {
        match self.client.process_json::<D>(self.client
            .http_client
            .post(url)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
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

impl<'a> MicrosoftAuthorization<'a, GrantCode> {
    /// Have a user go to this URL to generate the access_code
    /// Documented [Here](https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-auth-code-flow#request-an-authorization-code)
    ///
    /// [Successful Response](https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-auth-code-flow#successful-response)
    /// [Error Response](https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-auth-code-flow#error-codes-for-authorization-endpoint-errors)
    pub fn generate_login_url(&self) -> String {
        format!("{MICROSOFT_LOGIN_URL}/oauth2/v2.0/authorize?client_id={client_id}&response_type=code&redirect_uri={redirect_url}&scope={MICROSOFT_SCOPE}", client_id = self.client.auth_properties.azura_microsoft_client, redirect_url = self.internal.redirect_url)
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
        let authorization_url =
            format!("{MICROSOFT_LOGIN_URL}/oauth2/v2.0/token");
        let content = format!("client_id={client_id}&code={access_code}&scope={MICROSOFT_SCOPE}&grant_type=authorization_code&redirect_uri={redirect_url}",
                              client_id = self.client.auth_properties.azura_microsoft_client, redirect_url = self.internal.redirect_url);
        self.make_request(authorization_url, content).await
    }
}


