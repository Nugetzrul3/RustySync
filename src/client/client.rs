// Main logic for running file sync client
use crate::client::{
    file_watcher::watcher,
    db
};
use std::path::{ PathBuf };
use tokio::fs;
use std::error::Error;
use serde_json::json;
use tokio::io::AsyncWriteExt;
use crate::shared::utils;

pub async fn run_client(path: PathBuf) {
    println!("Starting client with path: {:?}", path);
    println!("Initialising DB...");
    if let Ok(conn) = db::init_db() {
        let watch_root = fs::canonicalize(&path).await.unwrap();
        if let Err(e) = watcher::watch_path(watch_root, &conn, &path).await {
            eprintln!("{:?}", e);
        }
    } else {
        eprintln!("Initialization of DB failed");
    }

}

pub async fn save_url(url: &str) -> Result<(), Box<dyn Error>> {
    let config_dir = match utils::get_config_path().await {
        Some(path) => path,
        None => {
            eprintln!("Error finding config directory");
            return Err(Box::from("Error finding config directory"));
        }
    };
    let mut config_file = fs::File::options().create(true).write(true).open(config_dir.join("config.json")).await?;

    let config = json!({
        "url": url,
    });

    let config_string = serde_json::to_string_pretty(&config)?;

    config_file.write_all(config_string.as_bytes()).await?;

    Ok(())


}