// Main entry point for running client/server
mod client;
mod server;
mod shared;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let mut args = std::env::args();
    let _ = args.next();

    match args.next().as_deref() {
        Some("client") => {
            println!("Running client");
            let path = args.next().expect("A client path is required");
            client::run_client(PathBuf::from(path));
        }

        Some("server") => {
            let port: u16 = args
                .next()
                .expect("Port required")
                .parse()
                .expect("Port must be a number");

            println!("Starting server with port {}", port);
            if let Err(e) = server::start(port).await {
                eprintln!("Server error{:?}", e);
            }

        }

        _ => {
            println!("Usage: cargo run -- client [path] or cargo run -- server [port]");
        }
    }
}
