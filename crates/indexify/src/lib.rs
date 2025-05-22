mod utils;

use std::thread;
use std::time::{Duration, SystemTime};

use crossbeam_channel::{Receiver, Sender};
use tracing::{Level, debug, info, span};
use tracing_subscriber::field::debug;
use utils::TANTIVY_INDEX;
use vaultify::VAULTIFY;

use facade::component::anything_item::Something;

pub fn index_files(path: &str) {
    let files = utils::get_files(path).unwrap();

    let mut conter: i64 = 0;
    debug!("begin indexing files from {}", path);
    for file in files {
        conter += 1;
        let file = file.unwrap();
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

pub fn init_index() {
    if VAULTIFY.get("indexed").unwrap() == "true" {
        info!("index already initialized, skipping");
        return;
    } else {
        let start = SystemTime::now();
        index_files("/Users/kxyang/Personal");
        let duration = start.elapsed().unwrap();
        VAULTIFY.set("indexed", "true".to_string()).unwrap();
        info!(
            "index initialized successfully in {} seconds",
            duration.as_secs()
        );
    }
}

pub fn init_service(request_reciver: Receiver<String>, data_sender: Sender<Vec<Something>>) {
    thread::spawn(move || {
        let span = span!(Level::DEBUG, "index service thread");
        let _enter = span.enter();
        init_index();
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
        println!("results: {:?}", results);
        assert!(!results.is_empty());
    }
}
