use std::fmt::Display;

use crate::{APIClient, Error};
use reqwest::header::CONTENT_TYPE;
use reqwest::Body;
use serde::{ser::SerializeStruct, Deserialize, Serialize};
static LOGIN_WITH_XBOX: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
#[derive(Deserialize, Debug)]
pub struct MinecraftLoginResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub username: String,
    pub roles: Vec<String>,
}

impl APIClient {
    /// Authenticate with Minecraft!
    /// After way to many queries to Microsoft you can finally get the Minecraft Bearer Token

    pub async fn authenticate_minecraft(
        &self,
        user_hash: impl AsRef<str>,
        xsts_token: impl AsRef<str>,
    ) -> Result<MinecraftLoginResponse, Error> {
        let identity_token = IdentityToken {
            user_hash: user_hash.as_ref(),
            xsts_token: xsts_token.as_ref(),
        };
        let body = serde_json::to_string(&identity_token)?;
        self.process_json::<MinecraftLoginResponse>(
            self.http_client
                .post(LOGIN_WITH_XBOX)
                .body(Body::from(body))
                .header(CONTENT_TYPE, "application/json"),
        )
        .await
    }
}
struct IdentityToken<'a> {
    user_hash: &'a str,
    xsts_token: &'a str,
}
impl Display for IdentityToken<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            user_hash,
            xsts_token,
        } = self;
        write!(f, "XBL3.0 x={user_hash};{xsts_token}")
    }
}
impl Serialize for IdentityToken<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("IdentityToken", 1)?;
        s.serialize_field("identityToken", &self.to_string())?;
        s.end()
    }
}
#[cfg(test)]
mod tests {
    use super::IdentityToken;

    #[test]
    pub fn test_serialize_identity_token() {
        let test = IdentityToken {
            user_hash: "MY_HASH",
            xsts_token: "MY_TOKEN",
        };
        let json = serde_json::to_string(&test).expect("Could not Serialize Identity Token");

        assert!(
            json.contains("XBL3.0 x=MY_HASH;MY_TOKEN"),
            "Does not Contain Token and Hash"
        );
        println!("{json}");
    }
}
