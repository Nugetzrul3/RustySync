// Helper utils for file syncing

use std::path::{ PathBuf };
use std::fs::File;
use std::io::{BufReader, Read};
use actix_web::{HttpRequest, HttpResponse};
use actix_web::http::header::Header;
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use argon2;
use argon2::PasswordVerifier;
use blake3;
use chrono::{DateTime, Utc};
use jsonwebtoken::{DecodingKey, Validation};
use serde_json::json;
use crate::shared::errors::AuthError;
use crate::shared::models::{AuthRequest, FileRow, UserAccessToken};

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
        let bytes_read = match reader.read(&mut buf) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("Read error: {:?}", e);
                return None;
            }
        };
        if bytes_read == 0 {
            break;
        }

        hasher.update(&buf[..bytes_read]);
    }

    Some(hasher.finalize().to_hex().to_string())
}

pub fn hash_filepath(filepath: &PathBuf) -> Option<String> {
    let file = File::open(filepath).ok()?;
    let mut reader = BufReader::new(file);
    let mut hasher = blake3::Hasher::new();

    let mut buf = [0u8; 8192];

    loop {
        let bytes_read = match reader.read(&mut buf) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("Read error: {:?}", e);
                return None;
            }
        };
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

pub fn not_found_error(error: String) -> HttpResponse {
    HttpResponse::NotFound().json(json!({ "status": "NOT_FOUND", "error": error }))
}

pub fn bad_request_error(error: String) -> HttpResponse {
    HttpResponse::BadRequest().json(json!({ "status": "BAD_REQUEST", "error": error }))
}

pub fn conflict_error(error: String) -> HttpResponse {
    HttpResponse::Conflict().json(json!({ "status": "CONFLICT", "error": error }))
}

pub fn authorization_error(error: String) -> HttpResponse {
    HttpResponse::Unauthorized().json(json!({ "status": "UNAUTHORIZED", "error": error }))
}

// Extract user information and return specific error
pub fn extract_user_info(request: &AuthRequest) -> Result<(String, String), AuthError> {
    let username = match request.username.clone() {
        Some(username) => username,
        None => return Err(AuthError::UsernameNotFound)
    };

    let password = match request.password.clone() {
        Some(password) => password,
        None => return Err(AuthError::PasswordNotFound)
    };

    Ok((username, password))

}

pub fn check_password(password: &String, password_hash: &argon2::PasswordHash) -> bool {
    match argon2::Argon2::default().verify_password(password.as_bytes(), password_hash) {
        Ok(_) => true,
        Err(_) => false
    }
}

pub fn validate_token(request: &HttpRequest) -> Result<UserAccessToken, HttpResponse> {
    let auth = match Authorization::<Bearer>::parse(request) {
        Ok(auth) => auth,
        Err(e) => {
            eprintln!("Parse error {}", e);
            return Err(internal_server_error(e.to_string()));
        }
    };

    let auth_token = auth.as_ref().token();

    let user_decoded = match jsonwebtoken::decode::<UserAccessToken>(
        auth_token,
        &DecodingKey::from_secret(std::env::var("JWT_SECRET").unwrap().as_ref()),
        &Validation::default(),
    ) {
        Ok(decoded) => decoded,
        Err(e) => {
            eprintln!("JSON Web token authentication failed, {}", e);
            return Err(authorization_error(e.to_string()))
        }
    };

    Ok(user_decoded.claims)
}
