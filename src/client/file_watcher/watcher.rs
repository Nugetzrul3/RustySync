// Core logic for file watching

use notify::{RecommendedWatcher, RecursiveMode, Watcher, Result, Config, EventKind};
use std::path::{ PathBuf };
use std::sync::mpsc::channel;
use std::time::{Duration, Instant, SystemTime};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use crate::client::utils;
use crate::client::db;

pub fn watch_path(watch_root: PathBuf, conn: &Connection) -> Result<()> {
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

                        std::thread::sleep(Duration::from_millis(100));

                        match &event.kind {
                            EventKind::Create(_) | EventKind::Modify(_) => {
                                if let Some(hash) = utils::hash_file(&path) {
                                    println!("File {} at {:?} with hash {}",
                                    if matches!(event.kind, EventKind::Modify(_)) { "Modified" } else { "Created" },
                                    path, hash);

                                    println!("Checking DB entries");

                                    let file_path = utils::format_file_path(&path.to_str().unwrap().to_string());
                                    let file_rows = db::get_file_row(conn, &file_path).unwrap_or_else(|e| {
                                        eprintln!("Error getting file row: {}", e);
                                        Vec::new()
                                    });
                                    let last_modified = DateTime::<Utc>::from(SystemTime::now());
                                    if file_rows.len() > 0 {
                                        // A file exists in our db, lets update it
                                        let mut file_row = file_rows.get(0).unwrap().clone();
                                        if file_row.hash() != hash {
                                            file_row.set_hash(hash);
                                        }

                                        file_row.set_last_modified(last_modified);

                                        db::update_file_row(conn, file_row).unwrap_or_else(|e| {
                                            eprintln!("Error updating DB entries: {:?}", e);
                                        });

                                    } else {
                                        // This file doesnt exist, lets create an entry

                                        let new_file_row = utils::convert_to_file_row(
                                            file_path,
                                            hash,
                                            last_modified
                                        );

                                        db::insert_file_row(conn, new_file_row).unwrap_or_else(|e| {
                                            eprintln!("Failed to insert new: {:?}", e);
                                        })
                                    }

                                } else {
                                    println!("Failed to hash file");
                                }
                            }

                            EventKind::Remove(_) => {
                                let file_path = utils::format_file_path(&path.to_str().unwrap().to_string());
                                db::remove_file_row(conn, &file_path).unwrap_or_else(|e| {
                                    eprintln!("Failed to remove file: {:?}", e);

                                });
                                println!("Removed: {:?}", path);
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
