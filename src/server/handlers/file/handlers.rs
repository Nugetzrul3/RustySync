use std::collections::HashMap;
use std::path::{PathBuf, Component::ParentDir};
use std::sync::Mutex;
use actix_multipart::Multipart;
// File upload and download handlers
use actix_web::{web, Responder};
use futures_util::{StreamExt, TryStreamExt};
use rusqlite::Connection;
use serde_json::json;
use tokio::fs;
use sanitize_filename;
use tokio::io::AsyncWriteExt;
use std::time::SystemTime;
use chrono::{DateTime, Utc };
use crate::shared::{
    models::FileRequest,
    utils
};
use crate::server::db;
use crate::server::handlers::auth::auth_extractor::AuthUser;

pub async fn files(auth: AuthUser, conn: web::Data<Mutex<Connection>>) -> impl Responder {
    let conn = conn.lock().unwrap();
    let user = auth.0;

    match db::get_files(&conn, &user.sub) {
        Ok(mut files) => {

            for file in &mut files {
                let stripped_path = PathBuf::from(file.path()).strip_prefix(format!("uploads/{}/", &user.sub).as_str()).unwrap().to_path_buf();
                file.set_path(stripped_path.to_string_lossy().to_string());
            }

            utils::okay_response(Some(json!(files)))
        },
        Err(e) => {
            eprintln!("{:?}", e);
            utils::internal_server_error(e.to_string())
        }
    }

}

pub async fn file(auth: AuthUser, query: web::Query<FileRequest>, conn: web::Data<Mutex<Connection>>) -> impl Responder {
    let conn = conn.lock().unwrap();
    let query = query.into_inner();
    let user = auth.0;

    let path = match query.path() {
        Some(path) => path,
        None => {
            eprintln!("Path not in request");
            return utils::bad_request_error(String::from("No path in request"));
        }
    };

    let mut root_path = PathBuf::from(format!("uploads/{}", user.sub));
    root_path.push(path);
    let formatted_path = utils::format_file_path(&root_path.to_string_lossy().to_string());

    let file_rows = match db::get_file(&conn, &formatted_path, &user.sub) {
        Ok(file_rows) => file_rows,
        Err(e) => {
            eprintln!("Error fetching file");
            return utils::internal_server_error(e.to_string());
        }
    };

    if file_rows.len() > 0 {
        let mut file = file_rows.get(0).unwrap().clone();
        let stripped_path = PathBuf::from(file.path()).strip_prefix(format!("uploads/{}/", &user.sub).as_str()).unwrap().to_path_buf();
        file.set_path(stripped_path.to_string_lossy().to_string());
        utils::okay_response(Some(json!(file)))

    } else {
        utils::not_found_error("File not found".to_string())
    }

}

pub async fn upload(auth: AuthUser, mut payload: Multipart, conn: web::Data<Mutex<Connection>>) -> impl Responder {
    let conn = conn.lock().unwrap();
    let mut files_success: HashMap<String, String> = HashMap::new();
    let mut files_failure: HashMap<String, String> = HashMap::new();
    let mut last_modified_map: HashMap<String, DateTime<Utc>> = HashMap::new();
    let mut file_path_map: HashMap<String, String> = HashMap::new();
    let username = auth.0.sub;

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
                    files_failure.insert(filename.to_string(), e.to_string());
                }
            }

            continue;

        } else if field_name.starts_with("path_") {
            let filename = field_name.strip_prefix("path_").unwrap();
            let mut data = Vec::new();

            while let Some(chunk) = field.next().await {
                data.extend_from_slice(&chunk.unwrap());
            }

            let value_str = String::from_utf8_lossy(&data).trim().to_string();
            file_path_map.insert(filename.to_string(), value_str);

            continue;

        } else if field_name.starts_with("file_") {
            let filename = if let Some(name) = cd.get_filename() {
                sanitize_filename::sanitize(name)
            } else {
                files_failure.insert(String::from(format!("Unknown file {}", cd.get_name().unwrap_or("N/A file"))), String::from("Missing filename"));
                continue;
            };

            let mut filepath = PathBuf::from(format!("uploads/{}", username));
            match fs::create_dir_all(&filepath).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error with directory creation: {:?}", e);
                    files_failure.insert(filename.clone(), e.to_string());
                    continue;
                }
            }

            let sent_path_raw = file_path_map.get(&filename).unwrap();
            if !sent_path_raw.is_empty() {
                let sent_path = PathBuf::from(sent_path_raw);

                if sent_path.is_absolute() || sent_path.components().any(|x| matches!(x, ParentDir)) {
                    files_failure.insert(filename.clone(), "Invalid path: must be relative and not contain '..'".into());
                    continue;
                }

                let final_path = filepath.join(&sent_path);

                match fs::create_dir_all(&final_path).await {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error with directory creation: {:?}", e);
                        files_failure.insert(filename.clone(), e.to_string());
                        continue;
                    }
                }

                filepath = final_path.join(&filename);

            } else {
                filepath.push(&filename);
            }

            println!("FINAL PATH {:?}", filepath);

            let file_query_path = utils::format_file_path(&filepath.to_string_lossy().to_string());

            let file_rows = match db::get_file(&conn, &file_query_path, &username.to_string()) {
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
                    eprintln!("File creation error {:?}", e);
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

                match db::insert_file(&conn, &file_row, &username.to_string()) {
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

pub async fn delete(auth: AuthUser, query: web::Query<FileRequest>, conn: web::Data<Mutex<Connection>>) -> impl Responder {
    let conn = conn.lock().unwrap();
    let query = query.into_inner();
    let user = auth.0;

    let path = match query.path() {
        Some(path) => path,
        None => {
            eprintln!("No path found in request");
            return utils::bad_request_error(String::from("No path found in request"));
        }
    };

    let mut root = PathBuf::from(format!("uploads/{}", user.sub));
    root.push(path);
    let filtered_path = utils::format_file_path(&root.to_string_lossy().to_string());

    let final_path = PathBuf::from(&filtered_path);

    if final_path.is_dir() {
        // Remove dir (and recursively find files to delete within)
        match fs::remove_dir_all(&final_path).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error with directory creation: {:?}", e);
                return utils::bad_request_error(String::from("Failed to delete directory"));
            }
        }

        utils::okay_response(None)
    } else {
        let file_row = match db::get_file(&conn, &filtered_path, &user.sub) {
            Ok(file_row) => file_row,
            Err(e) => {
                eprintln!("Error fetching file row: {}", e.to_string());
                return utils::internal_server_error(e.to_string());
            }
        };

        if file_row.is_empty() {
            utils::not_found_error(String::from("File not found"))
        } else {
            // first delete file from system
            match fs::remove_file(&filtered_path).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error deleting file: {:?}", e);
                    return utils::internal_server_error(e.to_string());
                }
            }

            // Then remove entry from db
            match db::remove_file(&conn, &filtered_path, &user.sub) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error deleting file: {:?}", e);
                    return utils::internal_server_error(e.to_string());
                }
            }

            utils::okay_response(None)
        }
    }

}
