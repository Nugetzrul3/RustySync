# RustySync
RustySync is a simple yet powerful file synchronisation tool written in Rust. 
It monitors a local folder and synchronises changes to a remote server like a lightweight Dropbox clone. 
Built to demonstrate efficient file I/O, concurrency, and network communication in Rust.

# Dependencies
- `clap` >= 4.5.32
- `notify` >= 8.0.0
- `blake3` >= 1.8.2
- `actix-web` >= 4.10.2
- `serde` >= 1.0.219
- `serde_json` >= 1.0.140
- `rusqlite` >= 0.34.0

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
`.\target\[build variant]\RustySync.exe server [port]`

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
     
- [ ] **Cross-platform Support**
  - Test on Linux, macOS, and Windows


### Future considerations
- Sync client: Retry on failure, parallelize uploads if possible (future consideration)
- Web server: Track historical versions of files

