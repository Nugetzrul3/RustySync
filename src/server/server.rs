// Main logic for hosting Actix-Web HTTP server
use actix_web::{ web, App, HttpServer };
use crate::server::handlers;
use crate::server::db;
use std::sync::Mutex;
use std::fs;

// basic server health check route
// main server startup
pub async fn start(port: u16) -> std::io::Result<()> {
    if let Ok(db_conn) = db::init_db() {
        let shared_conn = web::Data::new(Mutex::new(db_conn));
        match fs::create_dir_all("uploads") {
            Ok(_) => (),
            Err(e) => eprintln!("Error creating uploads dir: {:?}", e),
        }
        HttpServer::new(move || {
            App::new()
                .app_data(shared_conn.clone())
                .route("/health", web::get().to(handlers::health))
                .route("/files", web::get().to(handlers::files))
                .route("/file", web::get().to(handlers::file))
                .route("/upload", web::post().to(handlers::upload))
        })
            .bind(("127.0.0.1", port))?
            .run()
            .await
    } else {
        eprintln!("Failed to create database connection pool");
        Ok(())
    }
}
