mod utils;

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, thread};

use anyhow::Result;
use smol::channel::{Receiver, Sender};
use tracing::{Level, debug, error, info, span, warn};
use utils::{TANTIVY_INDEX, get_subfolders};
use vaultify::VAULTIFY;

use facade::component::anything_item::Something;

pub fn index_files(path: &str, remain_exclude_path: &Vec<String>, count_total: &mut u64) {
    let files = utils::get_files(path, remain_exclude_path).unwrap();
    debug!("begin indexing files from {}", path);
    for file in files {
        *count_total += 1;
        match file {
            Ok(file) => {
                TANTIVY_INDEX
                    .add(
                        file.file_name().to_str().expect("Failed to get file name"),
                        file.path().to_str().expect("Failed to get file path"),
                    )
                    .unwrap();
                if *count_total % 20000 == 0 {
                    VAULTIFY
                        .set("indexed_files", count_total.to_string())
                        .unwrap();
                    debug!("indexed {} files", count_total);
                }
            }
            Err(e) => {
                warn!("failed to get file type: {}", e);
                continue;
            }
        }
    }
    TANTIVY_INDEX.commit().unwrap();
    VAULTIFY
        .set("indexed_files", count_total.to_string())
        .unwrap();
    debug!("indexed {} files", count_total);
}

pub fn index_search(query: &str) -> Vec<Something> {
    TANTIVY_INDEX
        .search(query)
        .unwrap()
        .into_iter()
        .map(|mut item| {
            let path_str = item.path.clone().to_string();

            let name = path_str
                .rsplit('/')
                .find(|part| !part.is_empty())
                .unwrap_or("")
                .to_string();

            if let Ok(metadata) = fs::metadata(&path_str) {
                let class = name
                    .rsplit('.')
                    .next()
                    .filter(|ext| !ext.is_empty() && ext != &name)
                    .unwrap_or_else(|| if metadata.is_dir() { "folder" } else { "file" })
                    .to_string();

                let size = metadata.len() as f64;

                let last_modified_date = metadata
                    .modified()
                    .ok()
                    .and_then(|time| {
                        use time::OffsetDateTime;
                        let duration = time.duration_since(std::time::UNIX_EPOCH).ok()?;
                        OffsetDateTime::from_unix_timestamp(duration.as_secs() as i64).ok()
                    })
                    .unwrap_or_else(|| time::OffsetDateTime::now_utc())
                    .date();

                item.size = size;
                item.last_modified_date = last_modified_date;
                item.class = class.into();
            } else {
                let class = name
                    .rsplit('.')
                    .next()
                    .filter(|ext| !ext.is_empty() && ext != &name)
                    .unwrap_or("unknown")
                    .to_string();

                item.size = 0.0;
                item.last_modified_date = time::OffsetDateTime::now_utc().date();
                item.class = class.into();
            }

            item.name = name.into();
            item
        })
        .collect()
}

pub fn index_delete(path: &str) -> Result<()> {
    TANTIVY_INDEX.delete(path)?;
    Ok(())
}

pub fn index_add(path: &str) -> Result<()> {
    TANTIVY_INDEX.add(
        path.rsplit(|c| c == '/')
            .find(|part: &&str| !part.is_empty())
            .unwrap(),
        path,
    )?;
    Ok(())
}

pub fn index_commit() -> Result<()> {
    TANTIVY_INDEX.commit()?;
    Ok(())
}

pub fn index_list() -> Result<()> {
    TANTIVY_INDEX.list_all()?;
    Ok(())
}

pub fn init_index() -> Result<()> {
    if get_indexed_status()? {
        info!("index already initialized, skipping");
        return Ok(());
    } else {
        let start = SystemTime::now();
        let mut default_exclude_path = serde_json::from_str::<Vec<String>>(
            VAULTIFY.get("default_exclude_path").unwrap().as_str(),
        )?;
        let root_subfolder = get_subfolders("/");
        debug!("root_subfolder: {:?}", root_subfolder);

        let (excluded_paths, remaining_paths): (Vec<String>, Vec<String>) = root_subfolder
            .into_iter()
            .partition(|path| default_exclude_path.contains(path));

        for path in &excluded_paths {
            debug!("skipping path: {}", path);
        }

        default_exclude_path.retain(|path| !excluded_paths.contains(path));

        let mut count_percent = 0.0;
        let mut count_total: u64 = 0;
        let total_paths = remaining_paths.len();
        for (index, path) in remaining_paths.into_iter().enumerate() {
            debug!("processing path: {}", path);
            index_files(path.as_str(), &default_exclude_path, &mut count_total);
            count_percent = ((index + 1) as f64 / total_paths as f64) * 100.0;
            VAULTIFY.set("indexed_progress", count_percent.to_string())?;
        }
        debug!("completed processing all paths: {:.1}%", count_percent);

        let duration = start.elapsed()?;
        VAULTIFY.set("indexed", "true".to_string())?;
        VAULTIFY.set(
            "last_indexed",
            start
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string(),
        )?;
        VAULTIFY.set("indexed_files", get_num_docs().to_string())?;
        info!(
            "index initialized successfully in {} seconds",
            duration.as_secs()
        );
    }
    Ok(())
}

pub fn init_service(
    request_reciver: Receiver<String>,
    data_sender: Sender<Vec<Something>>,
) -> Result<()> {
    info!("Initializing index service...");
    thread::spawn(move || -> Result<()> {
        smol::block_on(async move {
            let span = span!(Level::DEBUG, "index service thread");
            let _enter = span.enter();
            init_index().expect("Failed to initialize index");
            while let Ok(data) = request_reciver.recv().await {
                let results = index_search(data.as_str());
                debug!("Search results: {:?}", results.len());
                if let Err(e) = data_sender.try_send(results) {
                    error!("Failed to send results: {}", e);
                }
            }
        });
        Ok(())
    });
    Ok(())
}

pub fn get_num_docs() -> u64 {
    TANTIVY_INDEX.get_num_docs()
}

pub fn get_indexed_status() -> Result<bool> {
    let current_version = env!("CARGO_PKG_VERSION");
    let stored_version = VAULTIFY.get("version")?;

    let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let last_indexed = VAULTIFY.get("last_indexed")?.parse::<u64>()?;
    let fifteen_days_in_seconds = 15 * 24 * 60 * 60;
    let reset_index_state = || -> Result<()> {
        let entries = [
            ("indexed", "false"),
            ("refresh", "false"),
            ("indexed_files", "0"),
            ("indexed_progress", "0.0"),
        ];
        VAULTIFY.batch_set(&entries)?;
        let tantivy_path = VAULTIFY.get("tantivy_path")?;
        if Path::new(&tantivy_path).exists() {
            fs::remove_dir_all(&tantivy_path).expect("Failed to remove: directory");
            debug!("Removed directory: {}", tantivy_path);
        }
        Ok(())
    };

    if current_time - last_indexed > fifteen_days_in_seconds {
        reset_index_state()?;
        return Ok(false);
    }

    if current_version != stored_version {
        VAULTIFY.set("version", current_version.to_string())?;
        reset_index_state()?;
        debug!(
            "reindexing due to version change: {} -> {}",
            current_version, stored_version
        );
        return Ok(false);
    }

    if VAULTIFY.get("refresh")? == "true" {
        reset_index_state()?;
        debug!("reindexing due to refresh flag being true");
        return Ok(false);
    }

    if VAULTIFY.get("indexed")? == "false" {
        reset_index_state()?;
        debug!("reindexing due to indexed flag being false");
        return Ok(false);
    }

    debug!("index is up to date");
    Ok(true)
}
