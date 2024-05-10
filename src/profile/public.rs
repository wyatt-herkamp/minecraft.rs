use std::str::{self, FromStr};

use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Deserialize;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub enum Model {
    Slim,
    Normal,
}
#[derive(Debug)]
pub struct GameProfileTextureMetadata {
    model: Model,
}
mod deserialize_metadata {
    use serde::de::{Error, Visitor};
    use serde::{Deserialize, Deserializer};
    use std::fmt::Formatter;
    use std::marker::PhantomData;

    pub struct MetadataVisitor<'de> {
        phantom: PhantomData<&'de ()>,
    }
    impl<'de> Visitor<'de> for MetadataVisitor<'de> {
        type Value = super::GameProfileTextureMetadata;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of textures")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            let mut model: Option<super::Model> = None;
            while let Some(key) = map.next_key::<&str>()? {
                match key {
                    "model" => {
                        let model_str: &str = map.next_value()?;
                        model = match model_str {
                            "slim" => Some(super::Model::Slim),
                            _ => Some(super::Model::Normal),
                        };
                    }
                    _ => {
                        let _: serde::de::IgnoredAny = map.next_value()?;
                    }
                }
            }
            let model = model.ok_or(Error::missing_field("model"))?;
            Ok(super::GameProfileTextureMetadata { model })
        }
    }

    impl<'de> Deserialize<'de> for super::GameProfileTextureMetadata {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(MetadataVisitor {
                phantom: PhantomData,
            })
        }
    }
}

impl GameProfileTextureMetadata {
    fn is_slim(&self) -> bool {
        self.model == Model::Slim
    }
}

#[derive(Deserialize, Debug)]
pub struct GameProfileTexture {
    url: String,
    metadata: Option<GameProfileTextureMetadata>,
}

impl GameProfileTexture {
    pub fn is_slim(&self) -> bool {
        self.metadata.as_ref().map(|m| m.is_slim()).unwrap_or(false)
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}
#[derive(Debug, Error)]
pub enum ProfileError {
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
    #[error(transparent)]
    UTF8Error(#[from] core::str::Utf8Error),
}

#[derive(Debug)]
pub struct GameProfileTextures {
    pub skin: GameProfileTexture,
    pub cape: Option<GameProfileTexture>,
}
impl<'de> TryFrom<&'de [u8]> for GameProfileTextures {
    type Error = ProfileError;

    fn try_from(value: &'de [u8]) -> Result<Self, Self::Error> {
        let base64 = STANDARD.decode(value)?;

        let json = str::from_utf8(base64.as_ref())?;
        return serde_json::from_str(json).map_err(ProfileError::from);
    }
}

mod deserialize_textures {
    use super::{GameProfileTexture, GameProfileTextures};
    use serde::de::{Error, Visitor};
    use serde::{Deserialize, Deserializer};
    use serde_json::Value;
    use std::fmt::Formatter;
    use std::marker::PhantomData;

    struct InnerTextures {
        skin: Option<GameProfileTexture>,
        cape: Option<GameProfileTexture>,
    }

    struct InnerTexturesVisitor<'de>(PhantomData<&'de ()>);

    impl<'de> Visitor<'de> for InnerTexturesVisitor<'de> {
        type Value = InnerTextures;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of textures")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            let mut skin: Option<GameProfileTexture> = None;
            let mut cape: Option<GameProfileTexture> = None;

            while let Some(key) = map.next_key::<&str>()? {
                match key {
                    "SKIN" => {
                        skin = Some(map.next_value::<GameProfileTexture>()?);
                    }
                    "CAPE" => {
                        cape = Some(map.next_value::<GameProfileTexture>()?);
                    }
                    _ => {
                        map.next_value::<Value>()?;
                    }
                }
            }
            Ok(InnerTextures { skin, cape })
        }
    }
    impl<'de> Deserialize<'de> for InnerTextures {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(InnerTexturesVisitor(PhantomData))
        }
    }
    struct TexturesVisitor<'de>(PhantomData<&'de ()>);
    impl<'de> Visitor<'de> for TexturesVisitor<'de> {
        type Value = GameProfileTextures;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of textures")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            let mut inner_textures: Option<InnerTextures> = None;
            while let Some(key) = map.next_key::<&str>()? {
                match key {
                    "textures" => inner_textures = Some(map.next_value()?),
                    _ => {
                        let _: serde::de::IgnoredAny = map.next_value()?;
                    }
                }
            }
            let Some(inner_textures) = inner_textures else {
                return Err(Error::missing_field("textures"));
            };

            Ok(GameProfileTextures {
                skin: inner_textures
                    .skin
                    .ok_or(Error::missing_field("textures.skin"))?,
                cape: inner_textures.cape,
            })
        }
    }
    impl<'de> Deserialize<'de> for GameProfileTextures {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(TexturesVisitor(PhantomData))
        }
    }
}

#[derive(Deserialize)]
struct GameProfileProperty<'p> {
    name: &'p str,
    value: &'p [u8],
}
#[derive(Debug)]
pub struct GameProfile {
    pub id: Uuid,
    pub name: String,
    pub textures: GameProfileTextures,
}
mod deserialize_game_profile {
    use serde::de::{Error, MapAccess, Visitor};
    use serde::{Deserialize, Deserializer};
    use std::fmt::Formatter;
    use std::marker::PhantomData;
    use uuid::Uuid;

    use super::{GameProfile, GameProfileProperty, GameProfileTextures};
    struct GameProfileVisitor<'de> {
        phantom: PhantomData<&'de ()>,
    }
    impl<'de> Visitor<'de> for GameProfileVisitor<'de> {
        type Value = GameProfile;
        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a struct of textures")
        }
        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut id: Option<Uuid> = None;
            let mut name: Option<String> = None;
            let mut textures: Option<GameProfileTextures> = None;

            while let Some(key) = map.next_key::<&str>()? {
                match key {
                    "id" => {
                        id = Some(map.next_value()?);
                    }
                    "name" => {
                        name = Some(map.next_value()?);
                    }
                    "properties" => {
                        let properties: Vec<GameProfileProperty> = map.next_value()?;
                        for property in properties {
                            match property.name {
                                "textures" => {
                                    textures = Some(
                                        GameProfileTextures::try_from(property.value)
                                            .map_err(Error::custom)?,
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {
                        let _: serde::de::IgnoredAny = map.next_value()?;
                    }
                }
            }
            let id = id.ok_or(Error::missing_field("id"))?;
            let name = name.ok_or(Error::missing_field("name"))?;
            let textures = textures.ok_or(Error::missing_field("textures"))?;
            Ok(GameProfile { id, name, textures })
        }
    }

    impl<'de> Deserialize<'de> for GameProfile {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(GameProfileVisitor {
                phantom: PhantomData,
            })
        }
    }
}
#[cfg(test)]
mod tests {
    #[tokio::test]
    pub async fn test() -> anyhow::Result<()> {
        let client = crate::test::setup();
        Ok(())
    }
}
