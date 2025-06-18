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
}

#[derive(Debug, Deserialize)]
pub struct FileRequest {
    pub path: String
}