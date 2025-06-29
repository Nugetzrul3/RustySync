use reqwest;
use serde_json::{
    json
};
use crate::shared::models::{ErrorResponse, LoginResponse, SuccessResponse};
use tokio::fs::File;
use std::error::Error;
use tokio::io::AsyncWriteExt;

pub async fn login_user(username: &str, password: &str) -> Result<(), Box<dyn Error>> {
    println!("Logging into user {}", username);
    let client = reqwest::Client::new();

    let resp = client.post("http://localhost:1234/auth/login")
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

        let mut token_file = File::create("token.json").await?;

        let json_data = json!({
            "access_token": data.data.access_token,
            "refresh_token": data.data.refresh_token,
            "token_type": data.data.token_type
        });

        let json_data_string = serde_json::to_string_pretty(&json_data)?;

        token_file.write_all(json_data_string.as_bytes()).await?;


    } else {
        let data = resp.json::<ErrorResponse>().await?;
        return Err(data.error.into());
    }

    Ok(())

}

pub async fn register_user(username: &str, password: &str) -> Result<(), Box<dyn Error>> {
    println!("Registering user {} with {}", username, password);
    let client = reqwest::Client::new();

    let resp = client.post("http://localhost:1234/auth/register")
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