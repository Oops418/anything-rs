mod utils;

use std::thread;
use std::time::SystemTime;

use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use tracing::{Level, debug, info, span, warn};
use utils::{TANTIVY_INDEX, get_subfolders};
use vaultify::VAULTIFY;

use facade::component::anything_item::Something;

pub fn index_files(path: &str, remain_exclude_path: &Vec<String>) {
    let files = utils::get_files(path, remain_exclude_path).unwrap();
    let mut conter: i64 = 0;
    debug!("begin indexing files from {}", path);
    for file in files {
        conter += 1;
        match file {
            Ok(file) => {
                TANTIVY_INDEX
                    .add(
                        file.file_name().to_str().expect("Failed to get file name"),
                        file.path().to_str().expect("Failed to get file path"),
                    )
                    .unwrap();
                if conter % 3000 == 0 {
                    debug!("indexed {} files", conter);
                }
            }
            Err(e) => {
                warn!("failed to get file type: {}", e);
                continue;
            }
        }
    }
    TANTIVY_INDEX.commit().unwrap();
    debug!("indexed {} files", conter);
}

pub fn index_search(query: &str) -> Vec<Something> {
    TANTIVY_INDEX
        .search(query)
        .unwrap()
        .into_iter()
        .map(|mut item| {
            let name = item
                .path
                .clone()
                .to_string()
                .rsplit(|c| c == '/')
                .find(|part: &&str| !part.is_empty())
                .unwrap()
                .to_string();

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

pub fn init_index() -> Result<()> {
    if VAULTIFY.get("indexed").unwrap() == "true" {
        info!("index already initialized, skipping");
        return Ok(());
    } else {
        let start = SystemTime::now();
        let mut default_exclude_path = serde_json::from_str::<Vec<String>>(
            VAULTIFY.get("default_exclude_path").unwrap().as_str(),
        )?;
        let root_subfolder = get_subfolders("/");
        debug!("root_subfolder: {:?}", root_subfolder);

        for path in root_subfolder {
            debug!("will decide path: {}", path);
            if default_exclude_path.iter().any(|s: &String| s == &path) {
                debug!("skipping path: {}", path);
                default_exclude_path.retain(|s| s != &path);
                continue;
            }
            index_files(path.as_str(), &default_exclude_path);
        }
        let duration = start.elapsed().unwrap();
        VAULTIFY.set("indexed", "true".to_string()).unwrap();
        info!(
            "index initialized successfully in {} seconds",
            duration.as_secs()
        );
    }
    Ok(())
}

pub fn init_service(request_reciver: Receiver<String>, data_sender: Sender<Vec<Something>>) {
    thread::spawn(move || {
        let span = span!(Level::DEBUG, "index service thread");
        let _enter = span.enter();
        init_index().expect("Failed to initialize index");
        loop {
            let request_query = request_reciver.recv().unwrap();
            debug!("received request: {}", request_query);
            let results = index_search(request_query.as_str());
            data_sender.send(results).unwrap();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search() {
        let results = index_search("Cargo");
        assert!(!results.is_empty());
    }
}
