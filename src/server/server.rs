// Main logic for hosting Actix-Web HTTP server
use actix_web::{ web, App, HttpServer };
use crate::server::handlers::{ file, auth };
use crate::server::db;
use std::sync::Mutex;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use rustls::{
    crypto::aws_lc_rs,
    ServerConfig,
    pki_types::{
        PrivateKeyDer
    }
};
use rustls_pemfile;

// HTTPS setup
fn load_config() -> Option<ServerConfig> {
    aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    let mut certs_file = BufReader::new(File::open("certs/cert.pem").ok()?);
    let mut keys_file = BufReader::new(File::open("certs/key.pem").ok()?);

    let tls_certs = rustls_pemfile::certs(&mut certs_file)
        .collect::<Result<Vec<_>, _>>().ok()?;
    let mut tls_key = rustls_pemfile::pkcs8_private_keys(&mut keys_file)
        .collect::<Result<Vec<_>, _>>().ok()?;

    if tls_key.is_empty() {
        eprintln!("No TLS keys found");
        return None;
    }

    let tls_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(tls_certs, PrivateKeyDer::Pkcs8(tls_key.remove(0)))
        .ok()?;

    Some(tls_config)
}

// basic server health check route
// main server startup
pub async fn start(port: u16) -> io::Result<()> {
    let tls_config = load_config();
    if let Ok(db_conn) = db::init_db() {
        let shared_conn = web::Data::new(Mutex::new(db_conn));
        match fs::create_dir_all("uploads") {
            Ok(_) => (),
            Err(e) => eprintln!("Error creating uploads dir: {:?}", e),
        }
        let server = HttpServer::new(move || {
            App::new()
                .app_data(shared_conn.clone())
                .route("/health", web::get().to(file::health))
                .route("/files", web::get().to(file::files))
                .route("/file", web::get().to(file::file))
                .route("/upload", web::post().to(file::upload))
                .route("/delete", web::delete().to(file::delete))

                .route("/register", web::post().to(auth::register))
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
