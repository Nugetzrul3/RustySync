// Main logic for running file sync client
use crate::client::file_watcher::watcher::watch_path;
use std::path::{ PathBuf };
use std::fs;
use crate::client::db::init_db;

pub fn run_client(path: PathBuf) {
    println!("Starting client with path: {:?}", path);
    println!("Initialising DB...");
    if let Ok(conn) = init_db() {
        let watch_root = fs::canonicalize(&path).unwrap();
        if let Err(e) = watch_path(watch_root, &conn) {
            eprintln!("{:?}", e);
        }
    } else {
        eprintln!("Initialization of DB failed");
    }

}