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
    // First sync files
    println!("Syncing directory {:?}", watch_root);
    sync::sync(&watch_root, conn, init_dir).await;

    // Channel to receive file change events
    let (tx, rx) = channel();

    // Create and instantiate the watcher
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    watcher.watch(&watch_root, RecursiveMode::Recursive)?;

    println!("Watching for changes in {:?}", watch_root);

    // Implement debouncer:
    // A debouncer is a concept used in programming where we want to
    // filter out multiple calls to a function or event so that it
    // will only run once during a certain time threshold. In this
    // instance, we will implement one for the Modify event

    let debounce_time = Duration::from_millis(500);
    let mut last_event_times = HashMap::<PathBuf, Instant>::new();

    // Loop events that are being received in the channel
    for res in rx {
        match res {
            Ok(event) => {
                for path in event.paths {
                    if !utils::check_file_path(&path) {
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
                                let file = File::open(&path)?;
                                if let Some(hash) = utils::hash_file(&file) {
                                    println!("File {} at {:?} with hash {}",
                                    if matches!(event.kind, EventKind::Modify(_)) { "Modified" } else { "Created" },
                                    path, hash);

                                    let relative_path = match path.strip_prefix(&watch_root) {
                                        Ok(p) => p.to_path_buf(),
                                        Err(_) => {
                                            eprintln!("Failed to get relative path for {:?}", path);
                                            continue;
                                        }
                                    };

                                    root_path.push(&relative_path);
                                    let file_path = utils::format_file_path(&root_path.to_string_lossy().to_string());
                                    let root_dir = init_dir.to_string_lossy().to_string();
                                    let file_rows = db::get_file(conn, &file_path, &root_dir).unwrap_or_else(|e| {
                                        eprintln!("Error getting file row: {}", e);
                                        Vec::new()
                                    });
                                    let file_metadata = file.metadata()?;
                                    let last_modified = DateTime::<Utc>::from(file_metadata.modified()?);
                                    if file_rows.len() > 0 {
                                        // A file exists in our db, lets update it
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
                                                continue;
                                            }
                                        }

                                    } else {
                                        // This file doesnt exist, lets create an entry

                                        let new_file_row = utils::convert_to_file_row(
                                            file_path,
                                            hash,
                                            last_modified
                                        );

                                        db::insert_file(conn, &new_file_row, &root_dir).unwrap_or_else(|e| {
                                            eprintln!("Failed to insert new: {:?}", e);
                                        });

                                        let mut files = Vec::<FileRow>::new();
                                        files.push(new_file_row);

                                        match apis::file::upload_files(files).await {
                                            Ok(_) => {}
                                            Err(e) => {
                                                eprintln!("Error uploading files: {}", e);
                                                continue;
                                            }
                                        }
                                    }

                                    root_path.clear();

                                } else {
                                    println!("Failed to hash file");
                                }
                            }

                            EventKind::Remove(_) => {
                                let relative_path = match path.strip_prefix(&watch_root) {
                                    Ok(p) => p.to_path_buf(),
                                    Err(_) => {
                                        eprintln!("Failed to get relative path for {:?}", path);
                                        continue;
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
            Err(e) => println!("watch error: {:?}", e)
        }
    }

    Ok(())

}
