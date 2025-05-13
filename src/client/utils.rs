// Helper utils for file syncing

use std::path::PathBuf;

// Check if file path is valid
pub fn check_file_path(path: &PathBuf) -> bool {
    let invalid_suffixes = ["~", ".tmp", ".swp"];
    let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

    if invalid_suffixes.iter().any(|suffix| file_name.ends_with(suffix)) {
        return false;
    }

    true
}