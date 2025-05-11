# RustySync
RustySync is a simple yet powerful file synchronisation tool written in Rust. 
It monitors a local folder and synchronises changes to a remote server like a lightweight Dropbox clone. 
Built to demonstrate efficient file I/O, concurrency, and network communication in Rust.

# Dependencies
- `clap`
- `notify`
- `blake3`
- `actix-web`
- `serde` 
- `serde_json`
- `rusqlite`

# How to build
1. Clone this repository
2. Run `cargo build` (or `cargo build --release` if you want release build)

# Roadmap
- [ ] **File Watcher**
  - Watch a directory for file changes using `notify`
  - Detect `create`, `modify`, and `remove` events

- [ ] **File Hashing**
  - Use `blake3` to fingerprint file contents
  - Store hashes to detect if files are modified

- [ ] **Metadata Management**
  - Store file paths, hashes, and timestamps in SQLite via `rusqlite`
  - Serialize metadata as JSON using `serde_json`

- [ ] **HTTP Server (Actix-Web)**
  - Build a RESTful API to receive file uploads and metadata
  - Save received files and update server-side SQLite metadata

- [ ] **Sync Client**
  - Send file updates to the server when changes are detected
  - Retry on failure, parallelize uploads if possible

- [ ] **CLI Interface**
  - Use `clap` to build a user-friendly CLI
  - Support flags like `--sync`, `--watch`, `--server`, etc.
     
- [ ] **Cross-platform Support**
  - Test on Linux, macOS, and Windows

