// Main entry point for running client/server
mod client;
mod server;
mod shared;
use std::path::PathBuf;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    let mut args = std::env::args();
    let _ = args.next();
    dotenv().ok();

    match args.next().as_deref() {
        Some("client") => {

            match args.next().as_deref() {
                Some("start") => {
                    println!("Running client");
                    let path = args.next().expect("A client path is required");
                    client::run_client(PathBuf::from(path));
                }

                Some("login") => {
                    let username = args.next().expect("A client username is required");
                    let password = args.next().expect("A client password is required");

                    match client::login_user(username.as_str(), password.as_str()).await {
                        Ok(_) => {},
                        Err(e) => {
                            eprintln!("Failed to login user, {:?}", e);
                        }
                    }

                }

                Some("register") => {

                    let username = args.next().expect("A client username is required");
                    let password = args.next().expect("A client password is required");

                    match client::register_user(username.as_str(), password.as_str()).await {
                        Ok(_) => {},
                        Err(e) => {
                            eprintln!("Failed to register user, {:?}", e);
                        }
                    }

                }

                _ => {
                    print!("Invalid client argument")
                }
            }


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
