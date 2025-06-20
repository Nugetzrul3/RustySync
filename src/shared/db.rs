use std::path::Path;
use chrono::DateTime;
use rusqlite::{params, Connection};
use crate::shared::errors::DbError;
use crate::shared::models::FileRow;
use crate::shared::utils;

// Core logic for handling SQLite DB interactions
pub fn init_db(server: bool) -> Result<Connection, DbError> {
    let db_path: &Path = if server {
        Path::new("server.db")
    } else {
        Path::new("client.db")
    };
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

pub fn get_file(conn: &Connection, path: &String) -> Result<Vec<FileRow>, DbError> {
    let mut statement = conn.prepare(
        "SELECT path, hash, last_modified FROM files WHERE path=?1"
    )?;

    let mut rows = statement.query(params![path])?;

    let mut file_rows: Vec<FileRow> = Vec::new();
    while let Some(row) = rows.next()? {
        // Converts database string rfc time to DateTime object
        let last_modified = DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)?;
        let file_row: FileRow = utils::convert_to_file_row(
            row.get(0)?,
            row.get(1)?,
            last_modified.to_utc()
        );

        file_rows.push(file_row);
    }

    Ok(file_rows)
}

pub fn update_file_row(conn: &Connection, file_row: FileRow) -> Result<(), DbError> {
    let mut statement = conn.prepare(
        "UPDATE files SET last_modified=?1, hash=?2 WHERE path=?3"
    )?;

    statement.execute(params![file_row.last_modified().to_rfc3339(), file_row.hash(), file_row.path()])?;
    Ok(())
}

pub fn insert_file(conn: &Connection, file: &FileRow) -> Result<(), DbError> {
    let mut statement = conn.prepare(
        "INSERT INTO files(path, hash, last_modified)\
            VALUES (?1, ?2, ?3)"
    )?;

    statement.execute(params![file.path(), file.hash(), file.last_modified().to_rfc3339()])?;
    Ok(())
}

pub fn remove_file(conn: &Connection, path: &String) -> Result<(), DbError> {
    let mut statement = conn.prepare(
        "DELETE FROM files WHERE path=?1"
    )?;

    statement.execute(params![path])?;

    Ok(())
}
