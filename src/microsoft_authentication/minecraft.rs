use reqwest::Body;
use reqwest::header::CONTENT_TYPE;
use serde::Deserialize;
use serde_json::json;
use crate::{ACCEPT, APIClient, Error};


#[derive(Deserialize, Debug)]
pub struct MinecraftLoginResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub username: String,
    pub roles: Vec<String>,
}

impl APIClient{
    /// Authenticate with Minecraft!
    /// After way to many queries to Microsoft you can finally get the Minecraft Bearer Token

    pub async fn authenticate_minecraft<U, X>(
        &self,
        user_hash: U,
        xsts_token: X,
    ) -> Result<MinecraftLoginResponse, Error> where U: AsRef<str>, X: AsRef<str> {
        let authorization_url =
            format!("https://api.minecraftservices.com/authentication/login_with_xbox");
        let user_hash = user_hash.as_ref();
        let xsts_token = xsts_token.as_ref();
        let identity_token = format!("XBL3.0 x={user_hash};{xsts_token}");
        let value = json!({
        "identityToken": identity_token
        });


        self.process_json::<MinecraftLoginResponse>(self
            .http_client
            .post(authorization_url).body(Body::from(serde_json::to_string(&value).unwrap()))
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
        ).await
    }

}
