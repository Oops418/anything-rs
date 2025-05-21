mod utils;

use std::thread;
use std::time::{Duration, SystemTime};

use crossbeam_channel::{Receiver, Sender};
use utils::TANTIVY_INDEX;
use vaultify::VAULTIFY;
use vaultify::Vaultify;

use facade::component::anything_item::Something;

pub fn index_files(path: &str) {
    let files = utils::get_files(path).unwrap();

    let start = SystemTime::now();
    let mut conter: i64 = 0;

    tracing::debug!("begin indexing files from {}", path);
    for file in files {
        conter += 1;
        let file = file.unwrap();
        TANTIVY_INDEX
            .add(
                file.file_name().to_str().expect("Failed to get file name"),
                file.path().to_str().expect("Failed to get file path"),
            )
            .unwrap();
        if conter % 1000 == 0 {
            tracing::debug!("Indexed {} files", conter);
        }
    }
    TANTIVY_INDEX.commit().unwrap();
    let duration = start.elapsed().unwrap();
    tracing::debug!(
        "indexed {} files in {} seconds",
        conter,
        duration.as_millis()
    );
}

pub fn index_search(query: &str) -> Vec<Something> {
    TANTIVY_INDEX.search(query).unwrap()
}

pub fn init_index() {
    if VAULTIFY.get("indexed").unwrap() == "true" {
        tracing::debug!("index already initialized");
        return;
    } else {
        index_files("/Users/kxyang/Personal/CodeSpaces/anything-rs");
        VAULTIFY.set("indexed", "true".to_string()).unwrap();
        tracing::debug!("index initialized successfully");
    }
}

pub fn init_service(request_reciver: Receiver<String>, data_sender: Sender<Vec<Something>>) {
    thread::spawn(move || {
        loop {
            init_index();
            let request_query = request_reciver.recv().unwrap();
            let results = index_search(request_query.as_str());
            data_sender.send(results).unwrap();
            tracing::debug!("query finished");
        }
    });
}
