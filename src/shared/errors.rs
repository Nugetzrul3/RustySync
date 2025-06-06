use std::fmt::Display;
use std::error::Error;
// Shared file for error handling
use rusqlite;

#[derive(Debug)]
pub enum DbError {
    SqliteError(rusqlite::Error),
    ChronoError(chrono::ParseError),
    Custom(String)
}

impl Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::SqliteError(e) => write!(f, "sqlite error: {}", e),
            DbError::ChronoError(e) => write!(f, "chrono parse error {}", e),
            DbError::Custom(e) => write!(f, "custom error {}", e)
        }
    }
}

impl Error for DbError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DbError::SqliteError(e) => Some(e),
            DbError::ChronoError(e) => Some(e),
            DbError::Custom(_) => None
        }
    }
}

impl From<rusqlite::Error> for DbError {
    fn from(err: rusqlite::Error) -> Self {
        DbError::SqliteError(err)
    }
}

impl From<chrono::ParseError> for DbError {
    fn from(err: chrono::ParseError) -> Self {
        DbError::ChronoError(err)
    }
}