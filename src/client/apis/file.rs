use std::fs::File;
use chrono::Utc;
use crate::shared::models::FileRow;

pub async fn delete_file(path: String) -> Result<(), reqwest::Error> {
    // To delete file
    Ok(())
}

pub async fn upload_file(path: String, file: &File) -> Result<(), reqwest::Error> {
    // Upload created/modified files
    Ok(())
}

pub async fn get_files() -> Result<Vec<FileRow>, reqwest::Error> {
    // return list of all metadata for files belonging to that user
    Ok(Vec::<FileRow>::new())
}

pub async fn get_file(path: String) -> Result<FileRow, reqwest::Error> {
    // Get specific file metadata
    Ok(FileRow::new(String::new(), String::new(), Utc::now()))
}
