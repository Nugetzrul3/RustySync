// Helper utils for file syncing

use std::path::{ PathBuf, Path };
use std::fs::File;
use std::io::{BufReader, Read};
use blake3;
use chrono::{DateTime, Utc};
use crate::shared::models::FileRow;

// Check if file path is valid
pub fn check_file_path(path: &PathBuf) -> bool {
    let invalid_suffixes = ["~", ".tmp", ".swp"];
    let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

    if invalid_suffixes.iter().any(|suffix| file_name.ends_with(suffix)) {
        return false;
    }

    true
}

pub fn hash_file(path: &Path) -> Option<String> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);
    let mut hasher = blake3::Hasher::new();

    let mut buf = [0u8; 8192];

    loop {
        let bytes_read = reader.read(&mut buf).ok()?;
        if bytes_read == 0 {
            break;
        }

        hasher.update(&buf[..bytes_read]);
    }

    Some(hasher.finalize().to_hex().to_string())
}

pub fn format_file_path(path: &String) -> String {
    path.replace("\\", "/").replace("./", "")
}

pub fn convert_to_file_row(path: String, hash: String, last_modified: DateTime<Utc>) -> FileRow {
    FileRow::new(
        path,
        hash,
        last_modified
    )
}