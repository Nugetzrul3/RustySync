use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use actix_multipart::Multipart;
// File upload and download handlers
use actix_web::{web, Responder};
use futures_util::{StreamExt, TryStreamExt};
use rusqlite::Connection;
use serde_json::json;
use crate::server::db;
use tokio::fs;
use sanitize_filename;
use tokio::io::AsyncWriteExt;
use std::time::SystemTime;
use chrono::{ DateTime, Utc };
use crate::shared::utils;

pub async fn health() -> impl Responder {
    utils::okay_response(None)
}

pub async fn files(conn: web::Data<Mutex<Connection>>) -> impl Responder {
    let conn = conn.lock().unwrap();
    match db::get_files(&conn) {
        Ok(files) => utils::okay_response(Some(json!(files))),
        Err(e) => {
            eprintln!("{:?}", e);
            utils::internal_server_error(e.to_string())
        }
    }

}

pub async fn upload(mut payload: Multipart, conn: web::Data<Mutex<Connection>>) -> impl Responder {
    let conn = conn.lock().unwrap();
    let mut files_success: HashMap<String, String> = HashMap::new();
    let mut files_failure: HashMap<String, String> = HashMap::new();
    let mut last_modified_map: HashMap<String, DateTime<Utc>> = HashMap::new();

    // Iterate over the fields of the multipart file upload
    while let Ok(Some(mut field)) = payload.try_next().await {
        let cd = match field.content_disposition() {
            Some(cd) => cd.clone(),
            None => {
                files_failure.insert(String::from(format!("Unknown {}", field.name().unwrap_or("N/A field"))), String::from("No content found"));
                continue;
            }
        };

        let field_name = cd.get_name().unwrap_or("");

        if field_name.starts_with("last_modified_") {
            let filename = field_name.strip_prefix("last_modified_").unwrap();
            let mut data = Vec::new();

            while let Some(chunk) = field.next().await {
                data.extend_from_slice(&chunk.unwrap());
            }

            let value_str = String::from_utf8_lossy(&data).trim().to_string();
            match DateTime::parse_from_rfc3339(&value_str) {
                Ok(dt) => {
                    last_modified_map.insert(filename.to_string(), dt.to_utc());
                }
                Err(e) => {
                    eprintln!("Failed to parse datetime for {}: {:?}", filename, e);
                }
            }

            continue;

        } else if field_name.starts_with("file_") {
            let filename = if let Some(name) = cd.get_filename() {
                sanitize_filename::sanitize(name)
            } else {
                files_failure.insert(String::from(format!("Unknown file {}", cd.get_name().unwrap_or("N/A file"))), String::from("Missing filename"));
                continue;
            };

            let mut filepath = PathBuf::from("files");
            match fs::create_dir_all(&filepath).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error with directory creation: {:?}", e);
                    files_failure.insert(filename.clone(), e.to_string());
                    continue;
                }
            }

            filepath.push(&filename);

            let file_query_path = utils::format_file_path(&filepath.to_string_lossy().to_string());

            let file_rows = match db::get_file(&conn, &file_query_path) {
                Ok(file_rows) => file_rows,
                Err(e) => {
                    eprintln!("Error with DB: {:?}", e);
                    files_failure.insert(filename.clone(), e.to_string());
                    continue;
                }
            };

            if !file_rows.is_empty() {
                files_failure.insert(filename.clone(), String::from("File already exists"));
                continue;
            }

            let mut f = match fs::File::create(&filepath).await {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("{:?}", e);
                    files_failure.insert(filename.clone(), e.to_string());
                    continue;
                }
            };
            let mut error_writing: bool = false;

            while let Some(chunk) = field.next().await {
                if let Ok(data) = chunk {
                    match f.write_all(&data).await {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Error with file: {:?}", e);
                            files_failure.insert(filename.clone(), e.to_string());
                            error_writing = true;
                            break;
                        }
                    }
                } else {
                    files_failure.insert(filename.clone(), String::from("Failed to read chunk"));
                    error_writing = true;
                    break;
                }
            }

            if error_writing {
                continue;
            }

            if let Some(hash) = utils::hash_filepath(&filepath) {
                let filtered_path = utils::format_file_path(&filepath.to_str().unwrap().to_string());
                let last_modified = match last_modified_map.get(&filename) {
                    Some(dt) => dt.clone(),
                    None => DateTime::<Utc>::from(SystemTime::now())
                };

                let file_row = utils::convert_to_file_row(
                    filtered_path,
                    hash,
                    last_modified,
                );

                match db::insert_file(&conn, &file_row) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("{:?}", e);
                        files_failure.insert(filename.clone(), e.to_string());
                        continue;
                    }
                }

                files_success.insert(filename.clone(), String::from("Success!"));
            } else {
                files_failure.insert(filename.clone(), String::from("Failed to hash file"));
            }

        } else {
            continue
        }

    }

    if files_failure.is_empty() && files_success.is_empty() {
        utils::bad_request_error(String::from("No files uploaded"))
    } else {
        utils::okay_response(
            Some(json!(
                {
                   "uploaded": files_success,
                    "failed": files_failure,
                }
            ))
        )
    }

}
