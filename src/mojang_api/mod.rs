use serde::Deserialize;
use uuid::Uuid;
use crate::APIClient;
use crate::error::Error;



#[cfg_attr(feature = "serialize_all", derive(serde::Serialize))]
#[derive(Deserialize, Debug)]
pub struct UserByUsernameResponse {
    pub name: String,
    pub id: Uuid,
}

#[cfg_attr(feature = "serialize_all", derive(serde::Serialize))]
#[derive(Deserialize, Debug)]
pub struct UsernameChangeList {
    pub name: String,
    #[serde(rename = "changedToAt")]
    pub changed_at: i64,
}

impl APIClient {
    pub async fn get_user_by_username<S: AsRef<str>>(&self, username: S) -> Result<UserByUsernameResponse, Error> {
        self.process_json(self.http_client.get(format!("https://api.mojang.com/users/profiles/minecraft/{}", username.as_ref()))).await
    }
    pub async fn get_username_change_history<S: AsRef<str>>(&self, uuid: S) -> Result<UserByUsernameResponse, Error> {
        self.process_json(self.http_client.get(format!("https://api.mojang.com/user/profiles/{}/names", uuid.as_ref()))).await
    }
}