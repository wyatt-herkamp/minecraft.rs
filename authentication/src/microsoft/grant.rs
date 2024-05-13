use url::Url;

use crate::{
    error::{BadResponseOrError, InternalError},
    utils::query_string_builder::QueryString,
    AuthProperties, AuthenticationClient, AuthorizationTokenResponse, MicrosoftAuthorization,
    MicrosoftAuthorizationType, MicrosoftError, MICROSOFT_LOGIN_URL, MICROSOFT_SCOPE,
};

/// Grant Code follows the Microsoft Authentication Schema [Here](https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-auth-code-flow)
/// Requires the redirect url to be set
#[derive(Debug, Clone)]
pub struct GrantCode {
    pub redirect_url: String,
}
impl From<String> for GrantCode {
    fn from(value: String) -> Self {
        Self {
            redirect_url: value,
        }
    }
}

impl GrantCode {
    /// Creates a new Grant Code Authentication
    pub fn new<S: Into<String>>(
        redirect_url: S,
        api_client: AuthenticationClient,
    ) -> MicrosoftAuthorization<GrantCode> {
        MicrosoftAuthorization {
            client: api_client,
            internal: GrantCode::from(redirect_url.into()),
        }
    }
}

impl MicrosoftAuthorizationType for GrantCode {}
impl MicrosoftAuthorization<GrantCode> {
    /// Have a user go to this URL to generate the access_code
    /// Documented [Here](https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-auth-code-flow#request-an-authorization-code)
    ///
    /// [Successful Response](https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-auth-code-flow#successful-response)
    /// [Error Response](https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-auth-code-flow#error-codes-for-authorization-endpoint-errors)
    pub fn generate_login_url(&self) -> Result<Url, InternalError> {
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
            .map_err(InternalError::URLParse)
    }
    /// Generates a Authorization Token from the access code
    /// The access code should not be saved. This should immediately be given to this function
    /// Documented Here [Here](https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-auth-code-flow#request-an-access-token-with-a-client_secret)
    /// # Error
    /// Error responds in two types. The value is network or parsing errors. The inner value is a Microsoft Error Response
    pub async fn get_authorization_token(
        &self,
        access_code: String,
    ) -> Result<AuthorizationTokenResponse, BadResponseOrError<MicrosoftError>> {
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
