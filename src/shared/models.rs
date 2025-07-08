use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Shared file for data type models
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileRow {
    path: String,
    hash: String,
    last_modified: DateTime<Utc>,
}

impl FileRow {
    pub fn new(path: String, hash: String, last_modified: DateTime<Utc>) -> Self {
        FileRow {
            path,
            hash,
            last_modified,
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn hash(&self) -> &str {
        &self.hash
    }

    pub fn last_modified(&self) -> DateTime<Utc> {
        self.last_modified
    }

    pub fn set_hash(&mut self, hash: String) {
        self.hash = hash;
    }

    pub fn set_last_modified(&mut self, last_modified: DateTime<Utc>) {
        self.last_modified = last_modified;
    }

    pub fn set_path(&mut self, path: String) {
        self.path = path;
    }
}


#[derive(Debug, Deserialize)]
pub struct FileRequest {
    path: Option<String>,
}

impl FileRequest {
    pub fn path(&self) -> &Option<String> {
        &self.path
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    refresh_token: String,
}

impl RefreshRequest {
    pub fn refresh_token(&self) -> &str {
        &self.refresh_token
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserRow {
    username: String,
    password: String,
}

impl UserRow {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}

// JWT claims
#[derive(Debug, Serialize, Deserialize)]
pub struct UserAccessToken {
    pub sub: String,
    pub exp: usize,
}

impl UserAccessToken {
    pub fn new(sub: String, exp: usize) -> Self {
        Self { sub, exp }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRefreshToken {
    pub sub: String,
    pub exp: usize,
}

impl UserRefreshToken {
    pub fn new(sub: String, exp: usize) -> Self {
        Self { sub, exp }
    }
}

#[derive(Debug, Deserialize)]
pub struct SuccessResponse {
    pub message: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub status: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginTokenData {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_at: usize
}

impl LoginTokenData {
    pub fn set_access_token(&mut self, access_token: String) {
        self.access_token = access_token;
    }

    pub fn set_expires_at(&mut self, expires_at: usize) {
        self.expires_at = expires_at;
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    pub data: LoginTokenData,
    pub message: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshResponse {
    pub data: RefreshData,
    pub message: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshData {
    pub username: String,
    pub access_token: String,
    pub token_type: String,
    pub expires_at: usize
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub url: String,
}



