mod utils;

use std::time::SystemTime;

use utils::TANTIVY_INDEX;

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

pub fn index_search(query: &str) {
    let index = TANTIVY_INDEX.search(query).unwrap();
    println!("{:?}", index);
}
