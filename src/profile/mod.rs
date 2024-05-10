mod public;
pub use public::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{APIClient, Error};

#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub struct ProfileSkin {
    pub id: String,
    pub state: String,
    pub url: String,
    pub variant: String,
}

#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub struct ProfileResponse {
    pub id: Uuid,
    pub name: String,
    pub skins: Vec<ProfileSkin>,
    // TODO parse this data
    pub capes: Vec<Value>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UUIDToName {
    id: Uuid,
    name: String,
}

impl APIClient {
    pub async fn get_uuid_by_username(&self, name: &str) -> Result<UUIDToName, Error> {
        let url = format!("https://api.mojang.com/users/profiles/minecraft/{name}");
        self.process_json(self.http_client.get(url)).await
    }
    pub async fn get_profile_by_id(&self, id: Uuid) -> Result<GameProfile, Error> {
        let url = format!(
            "https://sessionserver.mojang.com/session/minecraft/profile/{}",
            id.to_string()
        );
        self.process_json(self.http_client.get(url)).await
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use tracing::info;
    use uuid::Uuid;

    #[tokio::test]
    async fn get_uuid_test() -> anyhow::Result<()> {
        let client = crate::test::setup();

        let response = client.get_uuid_by_username("KingTux").await?;
        assert_eq!(
            response.id,
            Uuid::from_str("d087006bd72c4cdf924d6f903704d05c").unwrap()
        );
        info!(?response);
        Ok(())
    }
    #[tokio::test]
    async fn get_profile_test() -> anyhow::Result<()> {
        let client = crate::test::setup();

        let response = client
            .get_profile_by_id(Uuid::from_str("d087006bd72c4cdf924d6f903704d05c").unwrap())
            .await?;
        info!(?response);
        Ok(())
    }
}
