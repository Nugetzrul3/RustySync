// Main logic for running file sync client
use crate::client::file_watcher::source::watch_path;
use std::path::{ PathBuf };

pub fn run_client(path: PathBuf) {
    println!("Starting client with path: {:?}", path);
    if let Err(e) = watch_path(path) {
        eprintln!("{:?}", e);
    }
}