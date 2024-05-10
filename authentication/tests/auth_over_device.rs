use std::{env, fs::OpenOptions, io::Write, path::PathBuf};

use anyhow::Context;
use chrono::Duration;
use minecraft_authentication::{
    device::{CheckTokenResponse, DeviceCode},
    error::BadResponseOrError,
    AuthProperties, AuthenticationClient,
};
use reqwest::Client;
use tokio::time::sleep;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::test]
async fn test_login() -> anyhow::Result<()> {
    match dotenv::from_filename("authentication.test.env") {
        Ok(loaded) => {
            println!("Loaded Dot Env from {loaded:?}");
        }
        Err(err) => {
            println!("Could not load `authentication.test.env` {err}")
        }
    };
    tracing_subscriber::registry()
        .with(fmt::layer().pretty())
        .with(EnvFilter::from_default_env())
        .init();
    let client = AuthenticationClient::new(
        Client::builder()
            .user_agent("Minecraft.rs Test Client")
            .build()?,
        AuthProperties {
            azura_microsoft_client: get_env("AZURA_MICROSOFT_CLIENT")?,
        },
    );

    let device_code = DeviceCode::new(client.clone());

    let mut authorize_request = match device_code.create_authorize_request().await {
        Ok(ok) => ok,
        Err(BadResponseOrError::Error(err)) => return Err(err.into()),
        Err(BadResponseOrError::ResponseError(microsoft)) => {
            println!("{:#?}", microsoft);
            return Err(microsoft.into());
        }
    };

    println!("{:#?}", authorize_request.internal);

    let mut response = CheckTokenResponse::Pending;
    while let CheckTokenResponse::Pending = &response {
        sleep(Duration::seconds(10).to_std()?).await;
        response = authorize_request.check_if_ready().await?;
    }

    println!("{:#?}", response);
    let approved = match response {
        CheckTokenResponse::Approved(approved) => approved,
        other => {
            println!("Exiting {other:?}");
            return Ok(());
        }
    };
    let account_save = client.create_account(approved).await?;

    let as_str = serde_json::to_string_pretty(&account_save)?;
    let file = env::current_dir()?.join("login.test.json");
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .read(true)
        .open(file)?;
    file.write_all(as_str.as_bytes())?;
    println!("You are authenticated!!!");
    Ok(())
}

fn get_env(name: &str) -> anyhow::Result<String> {
    std::env::var(name).with_context(|| format!("Could not find {name} please add it to the authentication.test.env or to your running environment variables"))
}
