use std::fmt::Formatter;

// Shared file for error handling
#[derive(Debug, Clone)]
pub struct DbError;

impl Err for DbError {
    fn description(&self) -> &str {
        "database error"
    }
}