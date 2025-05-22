use std::path::Path;
use std::error::Error;
use chrono::DateTime;
use rusqlite::{Connection, params };
use crate::shared::models::FileRow;
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

pub fn insert_file_row(conn: &Connection, file_row: FileRow) -> Result<(), Box<dyn Error>> {
    let mut statement = conn.prepare(
        "INSERT INTO files(path, hash, last_modified)\
            VALUES (?1, ?2, ?3)"
        )?;

    statement.execute(params![file_row.path(), file_row.hash(), file_row.last_modified().to_rfc3339()])?;
    Ok(())
}

pub fn update_file_row(conn: &Connection, file_row: FileRow) -> Result<(), Box<dyn Error>> {
    let mut statement = conn.prepare(
        "UPDATE files SET last_modified=?1, hash=?2 WHERE path=?3"
    )?;

    statement.execute(params![file_row.last_modified().to_rfc3339(), file_row.hash(), file_row.path()])?;
    Ok(())
}

pub fn get_file_row(conn: &Connection, path: String) -> Result<Vec<FileRow>, Box<dyn Error>> {
    let mut statement = conn.prepare(
        "SELECT * FROM files WHERE path=?1"
    )?;

    println!("Executing SQL: {} with path = {}", "SELECT * FROM files WHERE path=?1", path);
    let mut rows = statement.query(params![path])?;

    let mut file_rows: Vec<FileRow> = Vec::new();
    while let Some(row) = rows.next()? {
        // Converts database string rfc time to DateTime object
        let last_modified = DateTime::parse_from_rfc3339(&row.get::<_, String>(0)?)?;
        let file_row: FileRow = FileRow::new(
            row.get(0)?,
            row.get(1)?,
            last_modified.to_utc()

        );

        file_rows.push(file_row);
    }

    Ok(file_rows)
}
