// Main logic for running file sync client
use crate::client::{
    file_watcher::watcher,
    db
};
use std::path::{ PathBuf };
use std::fs;

pub async fn run_client(path: PathBuf) {
    println!("Starting client with path: {:?}", path);
    println!("Initialising DB...");
    if let Ok(conn) = db::init_db() {
        let watch_root = fs::canonicalize(&path).unwrap();
        if let Err(e) = watcher::watch_path(watch_root, &conn, &path).await {
            eprintln!("{:?}", e);
        }
    } else {
        eprintln!("Initialization of DB failed");
    }

}