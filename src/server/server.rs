// Main logic for hosting Actix-Web HTTP server
use actix_web::{ web, App, HttpServer };
use crate::server::handlers::{ file, auth };
use crate::server::db;
use std::sync::Mutex;
use std::fs;
use crate::server::config_loader;
use std::io;
use rustls_pemfile;

// basic server health check route
// main server startup
pub async fn start(port: u16) -> io::Result<()> {
    if let Ok(db_conn) = db::init_db() {
        let shared_conn = web::Data::new(Mutex::new(db_conn));
        match fs::create_dir_all("uploads") {
            Ok(_) => (),
            Err(e) => eprintln!("Error creating uploads dir: {:?}", e),
        }
        let tls_config = config_loader::load_config();
        let server = HttpServer::new(move || {
            App::new()
                .app_data(shared_conn.clone())
                .route("/health", web::get().to(file::health))

                .route("/file/list", web::get().to(file::files))
                .route("/file/metadata", web::get().to(file::file))
                .route("/file/upload", web::post().to(file::upload))
                .route("/file/delete", web::delete().to(file::delete))

                .route("/auth/register", web::post().to(auth::register))
                .route("/auth/login", web::post().to(auth::login))
                .route("/auth/refresh", web::post().to(auth::refresh))
        });

        if let Some(config) = tls_config {
            println!("Starting server with HTTPS");
            server.bind_rustls_0_23(("127.0.0.1", port), config)?.run().await
        } else {
            println!("Starting server with HTTPS disabled");
            server.bind(("127.0.0.1", port))?.run().await
        }

    } else {
        eprintln!("Failed to create database connection pool");
        Ok(())
    }
}
