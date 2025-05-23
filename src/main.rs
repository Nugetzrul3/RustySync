// Main entry point for running client/server
mod client;
mod shared;

use client::run_client;
use std::path::PathBuf;

fn main() {
    let mut args = std::env::args();
    let _ = args.next();

    match args.next().as_deref() {
        Some("client") => {
            let path = args.next().expect("A client path is required");
            run_client(PathBuf::from(path));
        }

        Some("server") => {
            println!("Starting server");

        }

        _ => {
            println!("Usage: cargo run -- [client|server] [path]");
        }
    }
}
