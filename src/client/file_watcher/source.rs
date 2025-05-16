// Core logic for file watching

use notify::{RecommendedWatcher, RecursiveMode, Watcher, Result, Config, EventKind};
use std::path::{ PathBuf };
use std::sync::mpsc::channel;
use std::time::{ Duration, Instant };
use std::collections::HashMap;

use crate::client::utils;

pub fn watch_path(path: PathBuf) -> Result<()> {
    // Channel to receive file change events
    let (tx, rx) = channel();

    // Create and instantiate the watcher
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    watcher.watch(&path, RecursiveMode::Recursive)?;

    println!("Watching for changes in {:?}", path);

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
                                    println!("{} File created at {:?} with hash {}",
                                    if matches!(event.kind, EventKind::Modify(_)) { "Modified" } else { "Created" },
                                    path, hash);
                                } else {
                                    println!("Failed to hash file");
                                }
                            }

                            EventKind::Remove(_) => {
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
