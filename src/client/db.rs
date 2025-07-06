use std::path::Path;
use chrono::DateTime;
use rusqlite::{Connection, params };
use crate::shared::models::FileRow;
use crate::shared::errors::{ DbError };
use crate::shared::utils;
pub fn init_db() -> Result<Connection, DbError> {
    let db_path: &Path = Path::new("client.db");
    let conn: Connection = Connection::open(db_path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS files(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE,
            hash TEXT NOT NULL,
            last_modified TEXT NOT NULL,
            root_dir TEXT NOT NULL
        )",
        params![],
    )?;
    Ok(conn)
}

pub fn insert_file(conn: &Connection, file_row: &FileRow, root_dir: &String) -> Result<(), DbError> {
    let mut statement = conn.prepare(
        "INSERT INTO files(path, hash, last_modified, root_dir)\
            VALUES (?1, ?2, ?3, ?4)"
    )?;

    statement.execute(params![file_row.path(), file_row.hash(), file_row.last_modified().to_rfc3339(), root_dir])?;
    Ok(())
}

pub fn update_file(conn: &Connection, file_row: &FileRow, root_dir: &String) -> Result<(), DbError> {
    let mut statement = conn.prepare(
        "UPDATE files SET last_modified=?1, hash=?2 WHERE path=?3 AND root_dir=?4"
    )?;

    statement.execute(params![file_row.last_modified().to_rfc3339(), file_row.hash(), file_row.path(), root_dir])?;
    Ok(())
}

pub fn get_file(conn: &Connection, path: &String, root_dir: &String) -> Result<Vec<FileRow>, DbError> {
    let mut statement = conn.prepare(
        "SELECT path, hash, last_modified FROM files WHERE path=?1 AND root_dir=?2"
    )?;

    let mut rows = statement.query(params![path, root_dir])?;

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

pub fn remove_file(conn: &Connection, path: &String, root_dir: &String) -> Result<(), DbError> {
    let mut statement = conn.prepare(
        "DELETE FROM files WHERE path=?1 AND root_dir=?2"
    )?;

    statement.execute(params![path, root_dir])?;

    Ok(())
}

pub fn get_files(conn: &Connection, root_dir: &String) -> Result<Vec<FileRow>, DbError> {
    let mut statement = conn.prepare("SELECT path, hash, last_modified FROM files WHERE root_dir=?1")?;

    let mut rows = statement.query(params![root_dir])?;
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