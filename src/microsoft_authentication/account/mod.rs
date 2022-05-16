use std::ops::Add;
use chrono::{DateTime, Duration, Utc};
use log::debug;
use serde::{Serialize, Deserialize};
use crate::{APIClient, Error};
use crate::microsoft_authentication::microsoft::{AuthorizationTokenResponse, MicrosoftError, RefreshToken};
use crate::microsoft_authentication::minecraft::MinecraftLoginResponse;
use crate::microsoft_authentication::xbox::{XboxLiveResponse, XSTSError};

/// This structure gives an easy way to get the Minecraft token without you having to call all the functions again
/// it only stores the Refresh Token from Microsoft(Unknown Life Span), Xbox Token(14 days), Minecraft Token(24 hours)
/// We will not store the XSTS token because the life span is 24 hours. The same as the Minecraft Token
#[derive(Serialize, Deserialize, Debug)]
pub struct AccountSave {
    pub microsoft_token: MicrosoftToken,
    pub xbox: XboxUserSave,
    pub minecraft_save: MinecraftSave,
}

/// The Types an Account Load can create
pub enum AccountLoadError {
    /// Internal Error.
    InternalError(Error),
    /// Xbox Live Error
    XboxLiveError(XSTSError),
    /// Microsoft Error
    MicrosoftError(MicrosoftError),
}

impl From<Error> for AccountLoadError {
    fn from(error: Error) -> Self {
        AccountLoadError::InternalError(error)
    }
}

impl From<XSTSError> for AccountLoadError {
    fn from(error: XSTSError) -> Self {
        AccountLoadError::XboxLiveError(error)
    }
}

impl From<MicrosoftError> for AccountLoadError {
    fn from(error: MicrosoftError) -> Self {
        AccountLoadError::MicrosoftError(error)
    }
}

impl APIClient {
    /// Creates a based on the AAuthorization Response from Microsoft
    /// # Errors
    /// Read the docs on [AccountLoadError](AccountLoadError)
    pub async fn create_account(&self, auth: AuthorizationTokenResponse) -> Result<AccountSave, AccountLoadError> {
        debug!("Acquiring a Xbox Live Token");
        let xbox_save = self.authenticate_xbl(&auth.access_token).await??;
        debug!("Acquiring the XSTS token");
        let xsts = self.authenticate_xsts(&xbox_save.token).await??;
        debug!("Acquiring the Minecraft Token");
        let minecraft = self.authenticate_minecraft( xsts.get_user_hash_unsafe(), &xsts.token).await?;
        Ok(AccountSave {
            microsoft_token: auth.into(),
            xbox: xbox_save.into(),
            minecraft_save: minecraft.into(),
        })
    }
    /// Updates the account param
    /// Returns true if updates happened false if no update happened
    /// # Errors
    /// Read the docs on [AccountLoadError](AccountLoadError)
    pub async fn load_account(&self, account: &mut AccountSave) -> Result<bool, AccountLoadError> {
        let current_time = Utc::now();
        if account.minecraft_save.expires <= current_time {
            debug!("Minecraft Token Expired");
          if account.xbox.expires <= current_time {
                debug!("Xbox Token Expired.");
                debug!("Acquiring a access_token from Microsoft");
                let response = RefreshToken::new(&self).refresh_token(&account.microsoft_token.refresh_token).await??;
                account.microsoft_token.refresh_token = response.refresh_token;
                debug!("Acquiring a new Xbox Live Token");
                let ok = self.authenticate_xbl(&response.access_token).await??;
                account.xbox = ok.into();
            }
            debug!("Acquiring the XSTS token");
            let xsts = self.authenticate_xsts(&account.xbox.token).await??;
            debug!("Acquiring the Minecraft Token");
            let minecraft = self.authenticate_minecraft(xsts.get_user_hash_unsafe(), &xsts.token).await?;
            account.minecraft_save = minecraft.into();
            return Ok(true);
        }
        return Ok(false);
    }
}

/// The parts saved of the Microsoft Token
#[derive(Serialize, Deserialize, Debug)]
pub struct MicrosoftToken {
    pub refresh_token: String,
}


impl From<AuthorizationTokenResponse> for MicrosoftToken {
    fn from(value: AuthorizationTokenResponse) -> Self {
        MicrosoftToken {
            refresh_token: value.refresh_token
        }
    }
}

/// Xbox User save
#[derive(Serialize, Deserialize, Debug)]
pub struct XboxUserSave {
    pub expires: DateTime<Utc>,
    pub token: String,
    pub user_hash: String,
}

impl From<XboxLiveResponse> for XboxUserSave {
    fn from(value: XboxLiveResponse) -> Self {
        let mut value = value;
        // Remove consumes the value
        let user_hash = value.display_claims.x_ui.remove(0);
        XboxUserSave {
            expires: value.not_after,
            token: value.token,
            user_hash: user_hash.user_hash,
        }
    }
}

/// The Minecraft Auth Token
#[derive(Serialize, Deserialize, Debug)]
pub struct MinecraftSave {
    /// Then it expires
    pub expires: DateTime<Utc>,
    /// Minecraft Token
    pub token: String,
}

impl From<MinecraftLoginResponse> for MinecraftSave {
    fn from(value: MinecraftLoginResponse) -> Self {
        let current = Utc::now().add(Duration::seconds(value.expires_in));
        MinecraftSave {
            expires: current,
            token: value.access_token,
        }
    }
}