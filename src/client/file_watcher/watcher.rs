// Core logic for file watching

use notify::{RecommendedWatcher, RecursiveMode, Watcher, Result, Config, EventKind};
use std::path::{ PathBuf };
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::fs::File;
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use crate::shared::utils;
use crate::client::{
    file_watcher::sync,
    db,
    apis
};
use async_std::task;
use crate::shared::models::FileRow;

pub async fn watch_path(watch_root: PathBuf, conn: &Connection, init_dir: &PathBuf) -> Result<()> {
    // Check if user has logged in yet
    let config_dir = match utils::get_config_path().await {
        Some(config_dir) => config_dir,
        None => {
            eprintln!("Error finding config path");
            return Ok(())
        }
    };

    if !config_dir.join("config.json").exists() {
        eprintln!("Config json does not exist. You probably haven't set a server URL yet (client set-url --url [server-url])");
        return Ok(())
    }

    if !config_dir.join("token.json").exists() {
        eprintln!("WARNING: token.json does not exist, please login/register first");
        return Ok(())
    }

    // First sync files
    println!("Syncing directory {:?}", watch_root);
    sync::sync(&watch_root, conn, init_dir).await;

    // Channel to receive file change events
    let (tx, rx) = channel();

    // Create and instantiate the watcher
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    watcher.watch(&watch_root, RecursiveMode::Recursive)?;

    println!("Watching for changes in {:?}", watch_root);

    let debounce_time = Duration::from_millis(500);
    let mut last_event_times = HashMap::<PathBuf, Instant>::new();

    for res in rx {
        match res {
            Ok(event) => {
                for path in event.paths {
                    if !utils::check_file_path_safe(&path) {
                        continue;
                    }

                    let now = Instant::now();
                    let should_process = match last_event_times.get(&path) {
                        Some(last_time) => now.duration_since(*last_time) > debounce_time,
                        None => true,
                    };

                    if should_process {
                        last_event_times.insert(path.clone(), now);

                        task::sleep(Duration::from_millis(100)).await;
                        let mut root_path = PathBuf::from(init_dir);

                        match &event.kind {
                            EventKind::Create(_) | EventKind::Modify(_) => {
                                match handle_file_create_modify(&path, &watch_root, &mut root_path, conn, init_dir, &event.kind).await {
                                    Ok(_) => {},
                                    Err(e) => {
                                        eprintln!("Error handling file create/modify for {:?}: {}", path, e);
                                        continue;
                                    }
                                }
                            }

                            EventKind::Remove(_) => {
                                match handle_file_remove(&path, &watch_root, &mut root_path, conn, init_dir).await {
                                    Ok(_) => {},
                                    Err(e) => {
                                        eprintln!("Error handling file remove for {:?}: {}", path, e);
                                        continue;
                                    }
                                }
                            }

                            _ => {
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Watch error: {:?}", e);
            }
        }
    }

    Ok(())
}

async fn handle_file_create_modify(
    path: &PathBuf,
    watch_root: &PathBuf,
    root_path: &mut PathBuf,
    conn: &Connection,
    init_dir: &PathBuf,
    event_kind: &EventKind
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(());
    }

    let file = match File::open(&path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open file {:?}: {}", path, e);
            return Ok(());
        }
    };

    let hash = match utils::hash_file(&file) {
        Some(hash) => hash,
        None => {
            eprintln!("Failed to hash file {:?}", path);
            return Ok(());
        }
    };

    println!("File {} at {:?} with hash {}",
             if matches!(event_kind, EventKind::Modify(_)) { "Modified" } else { "Created" },
             path, hash);

    let relative_path = match path.strip_prefix(&watch_root) {
        Ok(p) => p.to_path_buf(),
        Err(_) => {
            eprintln!("Failed to get relative path for {:?}", path);
            return Ok(());
        }
    };

    root_path.push(&relative_path);
    let file_path = utils::format_file_path(&root_path.to_string_lossy().to_string());
    let root_dir = init_dir.to_string_lossy().to_string();

    let file_rows = db::get_file(conn, &file_path, &root_dir).unwrap_or_else(|e| {
        eprintln!("Error getting file row: {}", e);
        Vec::new()
    });

    let file_metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(e) => {
            eprintln!("Failed to get file metadata for {:?}: {}", path, e);
            return Ok(());
        }
    };

    let last_modified = match file_metadata.modified() {
        Ok(modified) => DateTime::<Utc>::from(modified),
        Err(e) => {
            eprintln!("Failed to get modification time for {:?}: {}", path, e);
            return Ok(());
        }
    };

    if file_rows.len() > 0 {
        // Update existing file
        let mut file_row = file_rows.get(0).unwrap().clone();
        if file_row.hash() != hash {
            file_row.set_hash(hash);
        }
        file_row.set_last_modified(last_modified);

        db::update_file(conn, &file_row, &root_dir).unwrap_or_else(|e| {
            eprintln!("Error updating DB entries: {:?}", e);
        });

        match apis::file::upload_files(file_rows).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error uploading files: {}", e);
            }
        }
    } else {
        // Create new file entry
        let new_file_row = utils::convert_to_file_row(file_path, hash, last_modified);

        db::insert_file(conn, &new_file_row, &root_dir).unwrap_or_else(|e| {
            eprintln!("Failed to insert new: {:?}", e);
        });

        let mut files = Vec::<FileRow>::new();
        files.push(new_file_row);

        match apis::file::upload_files(files).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error uploading files: {}", e);
            }
        }
    }

    root_path.clear();
    Ok(())
}

async fn handle_file_remove(
    path: &PathBuf,
    watch_root: &PathBuf,
    root_path: &mut PathBuf,
    conn: &Connection,
    init_dir: &PathBuf
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let relative_path = match path.strip_prefix(&watch_root) {
        Ok(p) => p.to_path_buf(),
        Err(_) => {
            eprintln!("Failed to get relative path for {:?}", path);
            return Ok(());
        }
    };

    root_path.push(&relative_path);
    let file_path = utils::format_file_path(&root_path.to_string_lossy().to_string());
    let root_dir = init_dir.to_string_lossy().to_string();

    db::remove_file(conn, &file_path, &root_dir).unwrap_or_else(|e| {
        eprintln!("Failed to remove file: {:?}", e);
    });

    println!("Removed: {:?}", path);
    root_path.clear();

    match apis::file::delete_file(file_path).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error deleting file: {}", e);
        }
    }

    Ok(())
}