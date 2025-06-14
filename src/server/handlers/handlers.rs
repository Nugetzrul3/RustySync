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

    // Iterate over the fields of the multipart file upload
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();

        let filename = if let Some(name) = content_disposition.expect("No content found").get_filename() {
            sanitize_filename::sanitize(name)
        } else {
            continue;
        };

        let mut filepath = PathBuf::from("files");
        match fs::create_dir_all(&filepath).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{:?}", e);
                return utils::internal_server_error(e.to_string());
            }
        }
        filepath.push(filename);

        let mut f = match fs::File::create(&filepath).await {
            Ok(f) => f,
            Err(e) => {
                eprintln!("{:?}", e);
                return utils::internal_server_error(e.to_string());
            }
        };

        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();

            match f.write_all(&data).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{:?}", e);
                    return utils::internal_server_error(e.to_string());
                }
            }
        }

        // Add db entry
        return if let Some(hash) = utils::hash_file(&f.into_std().await) {
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
                    return utils::internal_server_error(e.to_string());
                }
            }

            utils::okay_response(None)
        } else {
            utils::internal_server_error(String::from("Failed to hash file"))
        }

    }

    utils::bad_request_error(String::from("No file was uploaded"))
}
