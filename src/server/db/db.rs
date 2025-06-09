use std::path::Path;
use chrono::DateTime;
use rusqlite::{params, Connection};
use crate::shared::errors::DbError;
use crate::shared::models::FileRow;
use crate::shared::utils;

// Core logic for handling SQLite DB interactions
pub fn init_db() -> Result<Connection, DbError> {
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

pub fn get_files(conn: &Connection) -> Result<Vec<FileRow>, DbError> {
    let mut statement = conn.prepare("SELECT path, hash, last_modified FROM files")?;

    let mut rows = statement.query(params![])?;
    let mut files: Vec<FileRow> = Vec::new();

    while let Some(row) = rows.next()? {
        let last_modified = DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)?;
        let file_row = utils::convert_to_file_row(
            row.get(0)?,
            row.get(1)?,
            last_modified.to_utc()
        );
        files.push(file_row);
    };

    Ok(files)

}
