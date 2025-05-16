use indexify;
use std::thread::sleep;
use std::time::Duration;
use vaultify;

fn main() {
    logger::init_logger();
    vaultify::init_vault();

    indexify::index_files("/Users/kxyang/Personal/CodeSpaces/anything-rs");
    sleep(Duration::from_secs(2));
    indexify::index_search("cargo");
}
