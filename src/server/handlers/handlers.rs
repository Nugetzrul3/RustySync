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
        Ok(files) => HttpResponse::Ok().json(json!({ "status": "OK", "files": files })),
        Err(e) => {
            eprintln!("{:?}", e);
            HttpResponse::InternalServerError().json(json!({ "status": "INTERNAL_SERVER_ERROR" }))
        }
    }

}

pub async fn upload(mut payload: Multipart, conn: web::Data<Mutex<Connection>>) -> impl Responder {
    let conn = conn.lock().unwrap();

    // Iterate over the fields of the multipart file upload
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();

        let filename = if let Some(name) = content_disposition.get_filename() {
            sanitize_filename::sanitize(name)
        } else {
            continue;
        };

        let mut filepath = PathBuf::from("files");
        fs::create_dir_all(&filepath).await?;
        filepath.push(filename);

        let mut f = fs::File::create(&filepath).await?;

        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();

            f.write_all(&data).await?;
        }

        // Add db entry
        if let Some(hash) = utils::hash_file(&filepath) {
            let file_row = utils::convert_to_file_row(
                filepath.to_str().unwrap().to_string(),
                hash,
                DateTime::<Utc>::from(SystemTime::now()),
            );

            db::insert_file(&conn, file_row)?;

            return HttpResponse::Ok().json(json!({
                "status": "OK", "message": "File uploaded successfully"
            }))

        } else {
            return HttpResponse::InternalServerError().json(json!({
                "status": "INTERNAL_SERVER_ERROR",
                "error": "Failed to hash file"
            }))
        }

    }

    HttpResponse::BadRequest().json(json!({
        "status": "BAD_REQUEST",
        "error": "No file was uploaded"
    }))
}
