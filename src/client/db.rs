use std::path::Path;
use std::error::Error;
use rusqlite::{Connection, params };
pub fn init_db() -> Result<(Connection), Box<dyn Error>> {
    let db_path: &Path = Path::new("files.db");
    let conn: Connection = Connection::open(db_path)?;
    conn.execute(
    "CREATE TABLE IF NOT EXISTS files(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE,
            hash TEXT NOT NULL,
            last_modified TEXT NOT NULL
        )",
    params![],
    )?;
    Ok(conn)
}