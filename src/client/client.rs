// Main logic for running file sync client
use crate::client::file_watcher::source::watch_path;
use std::path::{ PathBuf };
use crate::client::db::init_db;

pub fn run_client(path: PathBuf) {
    println!("Starting client with path: {:?}", path);
    println!("Initialising DB...");
    if let Ok(conn) = init_db() {
        if let Err(e) = watch_path(path, &conn) {
            eprintln!("{:?}", e);
        }
    } else {
        eprintln!("Initialization of DB failed");
    }

}