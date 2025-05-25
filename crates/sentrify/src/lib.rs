use anyhow::Result;
use crossbeam_channel::unbounded;
use indexify::{get_num_docs, index_add, index_commit, index_delete};
use notify::{
    Config, Error, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher, event::ModifyKind,
};
use std::{
    path::Path,
    thread::{self},
    time::Duration,
};
use tracing::{Level, debug, span, warn};
use vaultify::VAULTIFY;

pub fn init_service() {
    thread::spawn(move || {
        let span = span!(Level::DEBUG, "sentry service thread");
        let _enter = span.enter();
        if let Err(e) = guard(VAULTIFY.get("default_include_path").unwrap()) {
            warn!("guard error: {e:?}")
        }
    });
}

fn guard<P: AsRef<Path>>(path: P) -> Result<()> {
    let (event_sender, event_receiver) = unbounded::<Result<Event, Error>>();
    let mut watcher = RecommendedWatcher::new(event_sender, Config::default())?;
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;
    let default_exclude_paths =
        serde_json::from_str::<Vec<String>>(VAULTIFY.get("default_exclude_path").unwrap().as_str())
            .unwrap();

    loop {
        if let Ok(indexed_value) = VAULTIFY.get("indexed") {
            if indexed_value.as_str() == "true" {
                debug!("indexing complete, starting file monitoring");
                break;
            }
        }
        debug!("waiting for indexing to complete");
        thread::sleep(Duration::from_secs(2));
    }

    let mut count = 0;
    for res in event_receiver {
        match res {
            Ok(event) => match event.kind {
                EventKind::Create(_) => {
                    for path in event.paths {
                        let path_str = path.to_str().unwrap();

                        if default_exclude_paths
                            .iter()
                            .any(|exclude| path_str.starts_with(exclude))
                        {
                            debug!("index skip: {}", path_str);
                            continue;
                        }
                        count += 1;
                        index_add(path_str)?;
                    }
                }
                EventKind::Modify(kind) => {
                    if let ModifyKind::Name(_) = kind {
                        for path in event.paths {
                            let path_str = path.to_str().unwrap();

                            if default_exclude_paths
                                .iter()
                                .any(|exclude| path_str.starts_with(exclude))
                            {
                                debug!("index skip: {}", path_str);
                                continue;
                            }

                            if Path::new(path_str).exists() {
                                count += 1;
                                index_add(path_str)?;
                            } else {
                                count += 1;
                                index_delete(path_str)?;
                            }
                        }
                    }
                }
                _ => {}
            },
            Err(error) => warn!("watch error: {:?}", error),
        }
        if count == 200 {
            index_commit()?;
            VAULTIFY
                .set("indexed_files", get_num_docs().to_string())
                .unwrap();
            debug!("commit index batch: {}", count);
            count = 0;
        }
    }

    Ok(())
}
