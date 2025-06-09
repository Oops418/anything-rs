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
use tracing::{Level, debug, span, trace, warn};
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

pub fn guard<P: AsRef<Path>>(path: P) -> Result<()> {
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
                            trace!("index skip: {}", path_str);
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
                                trace!("index skip: {}", path_str);
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
                EventKind::Remove(_) => {
                    for path in event.paths {
                        let path_str = path.to_str().unwrap();

                        if default_exclude_paths
                            .iter()
                            .any(|exclude| path_str.starts_with(exclude))
                        {
                            trace!("index skip: {}", path_str);
                            continue;
                        }
                        count += 1;
                        index_delete(path_str)?;
                    }
                }
                _ => {}
            },
            Err(error) => warn!("watch error: {:?}", error),
        }
        if count == 1000 {
            index_commit()?;
            VAULTIFY
                .set("indexed_files", get_num_docs().to_string())
                .unwrap();
            trace!("commit index batch: {}", count);
            count = 0;
        }
    }

    Ok(())
}

#[cfg(test)]
#[cfg(feature = "mock")]
mod tests {
    use std::fs;
    use std::thread;
    use std::time::Duration;

    use super::*;
    use indexify::index_list;
    use indexify::{get_num_docs, index_search};
    use tempfile::TempDir;
    use vaultify::{VAULTIFY, Vaultify};

    #[test]
    #[ignore = "This test is unstable because issue #272 in indexify"]
    fn test_guard() {
        Vaultify::init_vault();
        logger::init_log();

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_path = temp_dir.path().to_str().unwrap();

        VAULTIFY.set("indexed", "true".to_string()).unwrap();

        let temp_path_clone = temp_path.to_string();
        thread::spawn(move || guard(&temp_path_clone));

        thread::sleep(Duration::from_millis(100));

        let mut file_paths = Vec::new();
        for i in 0..40 {
            let test_file_path = temp_dir.path().join(format!("test_file_{}.txt", i));
            fs::write(&test_file_path, format!("test content {}", i))
                .expect("Failed to create test file");
            file_paths.push(test_file_path);
        }
        thread::sleep(Duration::from_millis(1000));

        let doc_count = get_num_docs();
        assert!(
            doc_count == 40,
            "Expected at least 40 documents, found {}",
            doc_count
        );

        let search_results = index_search("4");
        assert!(
            !search_results.is_empty(),
            "Search should return results for indexed files"
        );

        let mut renamed_file_paths = Vec::new();
        for i in 0..10 {
            let old_path = &file_paths[i];
            let new_path = temp_dir
                .path()
                .join(format!("renamed_file_{}.txt", i * 101));

            fs::rename(old_path, &new_path).expect("Failed to rename test file");
            renamed_file_paths.push(new_path);
        }
        thread::sleep(Duration::from_millis(1000));
        index_commit().unwrap();
        thread::sleep(Duration::from_millis(1000));

        index_list().unwrap();
        let doc_count_after_rename = get_num_docs();
        assert_eq!(
            doc_count_after_rename, 40,
            "Document count should remain 40 after rename, found {}",
            doc_count_after_rename
        );
        thread::sleep(Duration::from_millis(1000));

        let search_results_renamed = index_search("303");
        assert!(
            !search_results_renamed.is_empty(),
            "Search should return results for renamed files"
        );

        for file_path in &file_paths[10..] {
            fs::remove_file(file_path).expect("Failed to delete test file");
        }

        for file_path in &renamed_file_paths {
            fs::remove_file(file_path).expect("Failed to delete renamed file");
        }

        thread::sleep(Duration::from_millis(1000));

        index_commit().unwrap();
        thread::sleep(Duration::from_millis(1000));

        let search_results_after_delete = index_search("15");
        let doc_count = get_num_docs();
        let _ = index_list();
        assert_eq!(doc_count, 0);
        assert!(
            search_results_after_delete.is_empty(),
            "Search results should be reduced after file deletion"
        );
    }
}
