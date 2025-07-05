use reqwest;
use serde_json::{
    json
};
use crate::shared::models::{ErrorResponse, LoginResponse, LoginTokenData, RefreshResponse, SuccessResponse};
use tokio::{
    fs,
    io::{
        AsyncWriteExt,
        AsyncReadExt
    },
};
use std::error::Error;
use crate::shared::utils;

pub async fn login_user(username: &str, password: &str) -> Result<(), Box<dyn Error>> {
    println!("Logging into user {}", username);
    let url = utils::load_url().await?;
    let client = reqwest::Client::new();

    let resp = client.post(format!("{}/auth/login", url))
        .json(&json!(
            {
                "username": username,
                "password": password,

            }
        ))
        .send()
        .await?;

    if resp.status().is_success() {
        let data = resp.json::<LoginResponse>().await?;
        println!("{}! Logged in. Saving tokens...", data.message);

        let mut token_file = fs::File::options().create(true).write(true).open("token.json").await?;

        let json_data = json!({
            "access_token": data.data.access_token,
            "refresh_token": data.data.refresh_token,
            "token_type": data.data.token_type
        });

        let json_data_string = serde_json::to_string_pretty(&json_data)?;

        token_file.write_all(json_data_string.as_bytes()).await?;
        token_file.flush().await?;


    } else {
        let data = resp.json::<ErrorResponse>().await?;
        return Err(data.error.into());
    }

    Ok(())

}

pub async fn register_user(username: &str, password: &str) -> Result<(), Box<dyn Error>> {
    println!("Registering user {} with {}", username, password);
    let url = utils::load_url().await?;
    let client = reqwest::Client::new();

    let resp = client.post(format!("{}/auth/register", url))
        .json(&json!(
            {
                "username": username,
                "password": password,
            }
        ))
        .send()
        .await?;

    if resp.status().is_success() {
        let data = resp.json::<SuccessResponse>().await?;
        println!("{} Registered {} to server database", data.message, username);
    } else {
        let data = resp.json::<ErrorResponse>().await?;
        return Err(data.error.into());
    }

    Ok(())

}

pub async fn refresh_user() -> Result<(), Box<dyn Error>> {
    println!("Refreshing Access token");
    let url = utils::load_url().await?;
    let client = reqwest::Client::new();
    let token_string = fs::read_to_string(&"token.json").await?;
    let mut token_file = fs::OpenOptions::new()
                            .write(true)
                            .read(true)
                            .open("token.json").await?;
    let mut token_json: LoginTokenData = serde_json::from_str(&token_string)?;

    let resp = client.post(format!("{}/auth/refresh", url))
        .json(&json!(
            {
                "refresh_token": token_json.refresh_token,
            }
        ))
        .send()
        .await?;


    if resp.status().is_success() {
        let data = resp.json::<RefreshResponse>().await?;
        let access_token = data.data.access_token;

        token_json.set_access_token(access_token);

        let json_string = serde_json::to_string_pretty(&token_json)?;

        token_file.write_all(json_string.as_bytes()).await?;

        Ok(())
    } else {
        let data = resp.json::<ErrorResponse>().await?;
        Err(data.error.into())
    }

}