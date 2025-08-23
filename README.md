# RustySync
RustySync is a simple yet powerful file synchronisation tool written in Rust. 
It monitors a local folder and synchronises changes to a remote server like a lightweight Dropbox clone. 
Built to demonstrate efficient file I/O, concurrency, and network communication in Rust.

# Dependencies
- `clap` >= 4.5.32 (with derive features)
- `notify` >= 8.0.0
- `blake3` >= 1.6.1
- `actix-web` >= 4.10.2 (with rustls-0_23 features)
- `actix-web-httpauth` >= 0.8.2
- `actix-multipart` >= 0.7.2
- `serde` >= 1.0.219 (with derive features)
- `serde_json` >= 1.0.140
- `tokio` >= 1.45.1 (with macros, rt-multi-thread, fs features)
- `rusqlite` >= 0.36.0 (with bundled features)
- `chrono` >= 0.4.41 (with serde features)
- `futures-util` >= 0.3.31
- `walkdir` >= 2.5.0
- `sanitize-filename` >= 0.6.0
- `jsonwebtoken` >= 9.3.1
- `argon2` >= 0.5.3
- `rustls` >= 0.23.28
- `rustls-pemfile` >= 2.2.0
- `dotenv` >= 0.15.0
- `directories-next` >= 2.0.0
- `reqwest` >= 0.12.20 (with json, multipart, stream features)
- `async-std` >= 1.13.1

# How to build
1. Clone this repository
2. Run `cargo build` (or `cargo build --release` if you want release build)

# Running client
`.\target\[build variant]\RustySync.exe client [options]`

### Options for client
`set-url --url [url]`: Sets the current API url to send requests too

`login --username [username] --password [password]`: Login to the server

`register --username [username] --password [password]`: Register an account with server

`start --path [path]`: Start the file watcher

To start the file watcher, you need to have set the API url as well as have logged in to the server to get an access token

# Running server
`.\target\[build variant]\RustySync.exe server --port [port]`

### Before running server, create .env
```text
JWT_SECRET = "VERY_STRONG_SECRET_HERE"
```

#### Optionally run server with HTTPS
You can also run the server using HTTPS. This can be done by creating a `certs` folder that has the `cert.pem` and `key.pem`. You can use a tool like `mkcert` to create these files

# Roadmap
- [x] **File Watcher**
  - Watch a directory for file changes using `notify`
  - Detect `create`, `modify`, and `remove` events

- [x] **File Hashing**
  - Use `blake3` to fingerprint file contents
  - Store hashes to detect if files are modified

- [x] **Metadata Management**
  - Store file paths, hashes, and timestamps in SQLite via `rusqlite`
  - Serialize metadata as JSON using `serde_json`

- [x] **Startup Sync**
  - Sync new files at startup using `walkdir`
  - Check hash and last modified time of existing files and update in DB

- [x] **HTTP Server (Actix-Web)**
  - Build a RESTful API to receive file uploads and metadata
  - Save received files and update server-side SQLite metadata
  - Securely authenticate users who wish to interact with the server

- [x] **Sync Client**
  - Send file updates to the server when changes are detected

- [x] **CLI Interface**
  - Use `clap` to build a user-friendly CLI
  - Support flags like `--sync`, `--watch`, `--server`, etc.
     
- [x] **Cross-platform Support**
  - Test on Linux, macOS, and Windows

- [x] **Build MVP releases**
  - For Linux, macOS, and Windows


### Future considerations
- Sync client: Retry on failure, parallelize uploads if possible (future consideration)
- Web server: Track historical versions of files
- Sync server with client: Server listens for file changes on uploads and client requests new files/modifications if any

