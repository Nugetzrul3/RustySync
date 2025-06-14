use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use actix_multipart::Multipart;
// File upload and download handlers
use actix_web::{web, HttpResponse, Responder};
use futures_util::{StreamExt, TryStreamExt};
use rusqlite::Connection;
use rusqlite::fallible_iterator::FallibleIterator;
use serde_json::json;
use crate::server::db;
use tokio::fs;
use sanitize_filename;
use tokio::io::AsyncWriteExt;
use std::time::SystemTime;
use chrono::{ DateTime, Utc };
use crate::shared::utils;

pub async fn health() -> impl Responder {
    HttpResponse::Ok().json(json!(
        {
            "status": "OK",
        }
    ))
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

    // Iterate over the fields of the multipart file upload
    while let Ok(Some(mut field)) = payload.try_next().await {
        println!("Field received: {:?}", field);

        let filename = if let Some(cd) = field.content_disposition() {
            if let Some(name) = cd.get_filename() {
                sanitize_filename::sanitize(name)
            } else {
                files_failure.insert(String::from("Unknown file"), String::from("Missing filename"));
                continue;
            }
        } else {
            files_failure.insert(String::from("Unknown file"), String::from("No content found"));
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
        println!("{:?}", &filename);

        filepath.push(&filename);

        println!("File path: {:?}", &filepath);

        let file_rows = match db::get_file(&conn, &filepath.to_string_lossy().to_string()) {
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

        println!("PATH: {:?}", filepath);

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

        if let Some(hash) = utils::hash_file(&f.into_std().await) {
            let filtered_path = utils::format_file_path(&filepath.to_str().unwrap().to_string());

            let file_row = utils::convert_to_file_row(
                filtered_path,
                hash,
                DateTime::<Utc>::from(SystemTime::now()),
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
// let content_disposition = field.content_disposition();
        //
        // let filename = if let Some(name) = content_disposition.expect("No content found").get_filename() {
        //     sanitize_filename::sanitize(name)
        // } else {
        //     continue;
        // };
        //
        // let mut filepath = PathBuf::from("files");
        // match fs::create_dir_all(&filepath).await {
        //     Ok(_) => {}
        //     Err(e) => {
        //         eprintln!("{:?}", e);
        //         return utils::internal_server_error(e.to_string());
        //     }
        // }
        // filepath.push(filename);
        //
        // let mut f = match fs::File::create(&filepath).await {
        //     Ok(f) => f,
        //     Err(e) => {
        //         eprintln!("{:?}", e);
        //         return utils::internal_server_error(e.to_string());
        //     }
        // };
        //
        // while let Some(chunk) = field.next().await {
        //     let data = chunk.unwrap();
        //
        //     match f.write_all(&data).await {
        //         Ok(_) => {}
        //         Err(e) => {
        //             eprintln!("{:?}", e);
        //             return utils::internal_server_error(e.to_string());
        //         }
        //     }
        // }
        //
        // // Add db entry
        // return if let Some(hash) = utils::hash_file(&f.into_std().await) {
        //     let filtered_path = utils::format_file_path(&filepath.to_str().unwrap().to_string());
        //
        //     let file_row = utils::convert_to_file_row(
        //         filtered_path,
        //         hash,
        //         DateTime::<Utc>::from(SystemTime::now()),
        //     );
        //
        //     match db::insert_file(&conn, &file_row) {
        //         Ok(_) => {}
        //         Err(e) => {
        //             eprintln!("{:?}", e);
        //             return utils::internal_server_error(e.to_string());
        //         }
        //     }
        //
        //     utils::okay_response(None)
        // } else {
        //     utils::internal_server_error(String::from("Failed to hash file"))
        // }

    // }
    //
    // utils::bad_request_error(String::from("No file was uploaded"))
// }
