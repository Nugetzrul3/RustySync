use std::sync::Mutex;
// File upload and download handlers
use actix_web::{web, HttpResponse, Responder};
use rusqlite::Connection;
use serde_json::json;
use crate::server::db;

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
