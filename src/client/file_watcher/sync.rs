use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use rusqlite::Connection;
// This module can walk entire directories recursively and efficently
use walkdir::WalkDir;
use chrono::{DateTime, Utc};
use crate::shared::utils;
use crate::client::{ db, apis };
use crate::shared::models::FileRow;

pub async fn sync(root: &PathBuf, conn: &Connection, init_dir: &PathBuf) {
    let mut file_paths: HashMap<String, u8> = HashMap::new();
    let root_dir = init_dir.to_string_lossy().to_string();
    let mut all_files: Vec<FileRow> = Vec::new();
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
        let query = db::get_file(conn, &file_path, &root_dir);

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
                file_paths.insert(file_path, 1);
                continue;
            }

            if file_row.last_modified() == last_modified_utc {
                file_paths.insert(file_path, 1);
                continue;
            }

            file_row.set_last_modified(last_modified_utc);
            file_row.set_hash(hash);

            all_files.push(file_row.clone());

            println!("Syncing file {}", path.display());
            db::update_file(conn, &file_row, &root_dir).unwrap_or_else(|e| eprintln!("Error updating file. {}", e));
        } else {
            // This file doesnt exist, lets create an entry

            let new_file_row = utils::convert_to_file_row(
                file_path.clone(),
                hash,
                last_modified_utc
            );

            all_files.push(new_file_row.clone());

            println!("New file {}", root_path.display());
            db::insert_file(conn, &new_file_row, &root_dir).unwrap_or_else(|e| {
                eprintln!("Failed to insert new: {:?}", e);
            })
        }

        file_paths.insert(file_path, 1);
        root_path.clear();

    }

    // go through db and clean deleted files
    let file_rows = db::get_files(conn, &root_dir).unwrap_or_else(|e| {
        eprintln!("Error getting file rows. {}", e);
        Vec::new()
    });

    let mut delete_paths: Vec<String> = Vec::new();

    if file_rows.len() > 0 {
        for file_row in file_rows.iter() {
            if let None = file_paths.get(file_row.path()) {
                // Proceed to delete path from db
                println!("Deleting file {}", file_row.path());
                db::remove_file(conn, &file_row.path().to_string(), &root_dir).unwrap_or_else(|e| eprintln!("Error deleting file. {}", e));
                delete_paths.push(file_row.path().to_string());
            }
        }
    }

    if all_files.len() > 0 {
        // Upload new/modified files
        match apis::file::upload_files(all_files).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error uploading files. {}", e);
            }
        }
    }

    if delete_paths.len() > 0 {
        // Delete files
        for path in delete_paths {
            match apis::file::delete_file(path.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error deleting file {}. {}", path, e);
                }
            }
        }
    }

}