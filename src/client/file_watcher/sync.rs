use std::fs::File;
use std::path::PathBuf;
use rusqlite::Connection;
// This module can walk entire directories recursively and efficently
use walkdir::WalkDir;
use crate::client::db;
use chrono::{DateTime, Utc};
use crate::shared::utils;

pub fn sync(root: &PathBuf, conn: &Connection, init_dir: &PathBuf) {
    // Loop through files
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|x| x.ok())
        .filter(|x| x.file_type().is_file())
    {
        let mut root_path = PathBuf::from(init_dir);
        let path = entry.path().to_path_buf();
        if !utils::check_file_path(&path) {
            continue;
        }
        let file = File::open(&path).unwrap();

        let hash = match utils::hash_file(&file) {
            Some(hash) => hash,
            None => {
                eprintln!("Error hashing file");
                continue;
            }
        };

        let last_modified = file.metadata().unwrap().modified().unwrap();

        let last_modified_utc = DateTime::<Utc>::from(last_modified);
        let relative_path = match path.strip_prefix(&root) {
            Ok(p) => p.to_path_buf(),
            Err(_) => {
                eprintln!("Failed to get relative path for {:?}", path);
                continue;
            }
        };
        root_path.push(&relative_path);
        let file_path = utils::format_file_path(&root_path.to_string_lossy().to_string());
        let query = db::get_file_row(conn, &file_path);

        let file_rows = match query {
            Ok(rows) => rows,
            Err(e) => {
                eprintln!("Error making query. {}", e);
                continue;
            }
        };

        if file_rows.len() > 0 {
            let mut file_row = file_rows.get(0).unwrap().clone();
            if file_row.hash() == hash {
                continue;
            }

            if file_row.last_modified() == last_modified_utc {
                continue;
            }

            file_row.set_last_modified(last_modified_utc);
            file_row.set_hash(hash);

            println!("Syncing file {}", path.display());
            db::update_file_row(conn, file_row).unwrap_or_else(|e| eprintln!("Error updating file. {}", e));
        } else {
            // This file doesnt exist, lets create an entry

            let new_file_row = utils::convert_to_file_row(
                file_path,
                hash,
                last_modified_utc
            );

            println!("New file {}", root_path.display());
            db::insert_file_row(conn, new_file_row).unwrap_or_else(|e| {
                eprintln!("Failed to insert new: {:?}", e);
            })
        }

        root_path.clear();

    }
}