use chrono::{DateTime, Duration, Local};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing::debug;
use url::Url;

use crate::{
    error::BadResponseOrError, utils::query_string_builder::QueryString, AuthProperties,
    AuthenticationClient, AuthorizationTokenResponse, MicrosoftAuthorization,
    MicrosoftAuthorizationType, MicrosoftError, MICROSOFT_LOGIN_URL, MICROSOFT_SCOPE,
};
static DEVICE_CODE_REQUEST_URL: Lazy<Url> = Lazy::new(|| {
    Url::parse(&format!("{MICROSOFT_LOGIN_URL}/oauth2/v2.0/devicecode"))
        .expect("Could not Parse the devicecode request url")
});
static GET_TOKEN_REQUEST_URL: Lazy<Url> = Lazy::new(|| {
    Url::parse(&format!("{MICROSOFT_LOGIN_URL}/oauth2/v2.0/token"))
        .expect("Could not Parse the get token request url")
});

pub const DEVICE_CODE_GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:device_code";
/// Device Code follows the Microsoft Authentication Schema [Here](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code)
/// This will not require a redirect URL for authentication
#[derive(Debug)]
pub struct DeviceCode;
impl DeviceCode {
    pub fn new(client: AuthenticationClient) -> MicrosoftAuthorization<DeviceCode> {
        MicrosoftAuthorization::<DeviceCode> {
            client,
            internal: DeviceCode,
        }
    }
}
impl MicrosoftAuthorizationType for DeviceCode {}

impl MicrosoftAuthorization<DeviceCode> {
    pub async fn create_authorize_request(
        self,
    ) -> Result<MicrosoftAuthorization<PendingApproval>, BadResponseOrError<MicrosoftError>> {
        let AuthProperties {
            azura_microsoft_client,
            ..
        } = &self.client.auth_properties;
        let query_string = QueryString::new_with_capacity(4)
            .add_str("client_id", azura_microsoft_client)
            .add_str("response_type", "code")
            .add_str("scope", MICROSOFT_SCOPE);

        let response = self
            .make_request::<PendingApprovalResponse>(DEVICE_CODE_REQUEST_URL.as_ref(), query_string)
            .await?;
        debug!(?response, "Moving to Pending Approval");

        Ok(MicrosoftAuthorization {
            client: self.client,
            internal: response.into(),
        })
    }
}
#[derive(Debug, Clone)]
pub enum CheckTokenResponse {
    Approved(AuthorizationTokenResponse),
    Declined,
    Pending,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PendingApprovalResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: Url,
    #[serde(with = "crate::utils::time::serde_duration_as_seconds")]
    pub expires_in: Duration,
    #[serde(with = "crate::utils::time::serde_duration_as_seconds")]
    pub interval: Duration,
    pub message: String,
}
#[derive(Debug, Clone)]
pub struct PendingApproval {
    pub response: PendingApprovalResponse,
    pub last_check: DateTime<Local>,
}
impl From<PendingApprovalResponse> for PendingApproval {
    fn from(value: PendingApprovalResponse) -> Self {
        Self {
            response: value,
            last_check: Local::now(),
        }
    }
}

impl MicrosoftAuthorizationType for PendingApproval {}
impl MicrosoftAuthorization<PendingApproval> {
    pub async fn check_if_ready(
        &mut self,
    ) -> Result<CheckTokenResponse, BadResponseOrError<MicrosoftError>> {
        let query_string = {
            let PendingApproval {
                response,
                last_check,
            } = &mut self.internal;

            let next_check = *last_check + response.interval;
            if next_check > Local::now() {
                return Ok(CheckTokenResponse::Pending);
            }

            *last_check = Local::now();
            let AuthProperties {
                azura_microsoft_client,
                ..
            } = &self.client.auth_properties;
            QueryString::new_with_capacity(4)
                .add_str("client_id", azura_microsoft_client)
                .add_str("grant_type", DEVICE_CODE_GRANT_TYPE)
                .add_string("device_code", response.device_code.clone())
        };
        let response = self
            .make_request::<AuthorizationTokenResponse>(
                GET_TOKEN_REQUEST_URL.as_ref(),
                query_string,
            )
            .await;

        match response {
            Ok(ok) => Ok(CheckTokenResponse::Approved(ok)),
            Err(err) => match err {
                BadResponseOrError::ResponseError(microsoft) => match microsoft.error.as_str() {
                    "authorization_pending" => Ok(CheckTokenResponse::Pending),
                    "authorization_declined" => Ok(CheckTokenResponse::Declined),
                    _ => Err(BadResponseOrError::ResponseError(microsoft)),
                },
                _ => Err(err),
            },
        }
    }
}
