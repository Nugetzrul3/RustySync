// Main entry point for running client/server
mod client;
mod server;
mod shared;
use std::path::PathBuf;
use dotenv::dotenv;
use clap::{ Parser, Subcommand };
use crate::client::apis;

// Commands
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    mode: Mode
}

#[derive(Subcommand, Debug)]
enum Mode {
    Server {
        #[arg(long)]
        port: u16,
    },

    Client {
        #[command(subcommand)]
        command: Commands
    }
}

#[derive(Subcommand, Debug)]
enum Commands {
    Register {
        #[arg(long)]
        username: String,

        #[arg(long)]
        password: String,
    },

    Login {
        #[arg(long)]
        username: String,

        #[arg(long)]
        password: String,
    },
    SetUrl {
        #[arg(long)]
        url: String,
    },

    Start {
        #[arg(long)]
        path: String
    },
    Refresh
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Use clap
    let cli = Cli::parse();

    match cli.mode {
        Mode::Server { port } => {
            println!("Starting server at port {}", port);
            if let Err(e) = server::start(port).await {
                eprintln!("Error starting server, {}", e);
            }
        }

        Mode::Client { command } => {
            match command {
                Commands::Register { username, password } => {
                    match apis::auth::register_user(&username, &password).await {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Error registering user, {}", e);
                        }
                    }
                }

                Commands::Login { username, password } => {
                    match apis::auth::login_user(&username, &password).await {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Error logging on user, {}", e);
                        }
                    }
                }

                Commands::Start { path } => {
                    let watch_path = PathBuf::from(path);
                    client::run_client(watch_path).await;
                }

                Commands::Refresh => {
                    match apis::auth::refresh_user().await {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Error refreshing user, {}", e);
                        }
                    }
                }
                Commands::SetUrl { url } => {
                    match client::save_url(url.as_str()).await {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Error saving url, {}", e);
                        }
                    }
                }
            }
        }
    }

}
