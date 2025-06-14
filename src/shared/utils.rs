// Helper utils for file syncing

use std::path::{ PathBuf };
use std::fs::File;
use std::io::{BufReader, Read};
use actix_web::HttpResponse;
use blake3;
use chrono::{DateTime, Utc};
use serde_json::json;
use crate::shared::models::FileRow;

// Check if file path is valid
pub fn check_file_path(path: &PathBuf) -> bool {
    if path.is_dir() {
        return false;
    }

    let invalid_suffixes = ["~", ".tmp", ".swp"];
    let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

    if invalid_suffixes.iter().any(|suffix| file_name.ends_with(suffix)) {
        return false;
    }

    true
}

pub fn hash_file(file: &File) -> Option<String> {
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

pub fn internal_server_error(error: String) -> HttpResponse {
    HttpResponse::InternalServerError().json(json!({ "status": "INTERNAL_SERVER_ERROR", "error": error }))
}

pub fn okay_response(data: Option<serde_json::Value>) -> HttpResponse {
    match data {
        Some(data) => HttpResponse::Ok().json(json!({ "status": "OK", "message": "Success", "data": data })),
        None => HttpResponse::Ok().json(json!({ "status": "OK", "message": "Success" })),
    }
}

pub fn bad_request_error(error: String) -> HttpResponse {
    HttpResponse::BadRequest().json(json!({ "status": "BAD_REQUEST", "error": error }))
}