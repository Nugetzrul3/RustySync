use std::path::Path;
use chrono::DateTime;
use rusqlite::{Connection, params };
use crate::shared::models::FileRow;
use crate::shared::errors::{ DbError };
pub fn init_db() -> Result<(Connection), DbError> {
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

pub fn insert_file_row(conn: &Connection, file_row: FileRow) -> Result<(), DbError> {
    let mut statement = conn.prepare(
        "INSERT INTO files(path, hash, last_modified)\
            VALUES (?1, ?2, ?3)"
        )?;

    statement.execute(params![file_row.path(), file_row.hash(), file_row.last_modified().to_rfc3339()])?;
    Ok(())
}

pub fn update_file_row(conn: &Connection, file_row: FileRow) -> Result<(), DbError> {
    let mut statement = conn.prepare(
        "UPDATE files SET last_modified=?1, hash=?2 WHERE path=?3"
    )?;

    statement.execute(params![file_row.last_modified().to_rfc3339(), file_row.hash(), file_row.path()])?;
    Ok(())
}

pub fn get_file_row(conn: &Connection, path: &String) -> Result<Vec<FileRow>, DbError> {
    let mut statement = conn.prepare(
        "SELECT * FROM files WHERE path=?1"
    )?;

    let mut rows = statement.query(params![path])?;

    let mut file_rows: Vec<FileRow> = Vec::new();
    while let Some(row) = rows.next()? {
        // Converts database string rfc time to DateTime object
        let last_modified = DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)?;
        let file_row: FileRow = FileRow::new(
            row.get(1)?,
            row.get(2)?,
            last_modified.to_utc()

        );

        file_rows.push(file_row);
    }

    Ok(file_rows)
}

pub fn remove_file_row(conn: &Connection, path: &String) -> Result<(), DbError> {
    let mut statement = conn.prepare(
        "DELETE FROM files WHERE path=?1"
    )?;

    statement.execute(params![path])?;

    Ok(())
}
