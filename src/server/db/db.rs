use std::path::Path;
use chrono::DateTime;
use rusqlite::{params, Connection};
use crate::shared::errors::DbError;
use crate::shared::models::FileRow;
use crate::shared::utils;
use argon2::{
    password_hash::{
        SaltString,
        rand_core::OsRng,
    },
    Argon2
};

// Core logic for handling SQLite DB interactions
pub fn init_db() -> Result<Connection, DbError> {
    let db_path: &Path = Path::new("server.db");
    let conn: Connection = Connection::open(db_path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS files(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE,
            hash TEXT NOT NULL,
            last_modified TEXT NOT NULL,
            username TEXT NOT NULL
        )",
        params![],
    )?;
    Ok(conn)
}

pub fn get_files(conn: &Connection, username: &String) -> Result<Vec<FileRow>, DbError> {
    let mut statement = conn.prepare("SELECT path, hash, last_modified FROM files WHERE username=?1")?;

    let mut rows = statement.query(params![username])?;
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

pub fn get_file(conn: &Connection, path: &String, username: &String) -> Result<Vec<FileRow>, DbError> {
    let mut statement = conn.prepare(
        "SELECT path, hash, last_modified FROM files WHERE path=?1 AND username=?2"
    )?;

    let mut rows = statement.query(params![path, username])?;

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

pub fn insert_file(conn: &Connection, file: &FileRow, username: &String) -> Result<(), DbError> {
    let mut statement = conn.prepare(
        "INSERT INTO files(path, hash, last_modified, username)\
            VALUES (?1, ?2, ?3, ?4)"
    )?;

    statement.execute(params![file.path(), file.hash(), file.last_modified().to_rfc3339(), username])?;
    Ok(())
}

pub fn remove_file(conn: &Connection, path: &String, username: &String) -> Result<(), DbError> {
    let mut statement = conn.prepare(
        "DELETE FROM files WHERE path=?1 AND username=?2"
    )?;

    statement.execute(params![path, username])?;

    Ok(())
}

pub fn register_user(conn: &Connection, username: &String, password: &String) -> Result<bool, DbError> {
    Ok(true)
}