use once_cell::sync::Lazy;
use redb::{Database, Error, TableDefinition};

static CONFIG_PATH: Lazy<&str> = Lazy::new(|| {
    tracing::debug!("Initializing config path...");
    "/Users/kxyang/Personal/CodeSpaces/anything-rs/anything.redb"
});

static TANTIVY_PATH: Lazy<&str> = Lazy::new(|| {
    tracing::debug!("Initializing tantivy path...");
    "/Users/kxyang/Personal/CodeSpaces/anything-rs/tantivy"
});

pub fn init_vault() {
    set("config_path", String::from(*CONFIG_PATH)).expect("Failed to set config_path");
    set("tantivy_path", String::from(*TANTIVY_PATH)).expect("Failed to set tantivy_path");
}

const TABLE: TableDefinition<&str, String> = TableDefinition::new("anything");

pub fn get(key: &str) -> Result<String, Error> {
    let db = Database::open(&*CONFIG_PATH)?;
    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(TABLE)?;
    let value = table.get(key)?.unwrap().value();
    Ok(String::from(value))
}

pub fn set(key: &str, value: String) -> Result<(), Error> {
    let db = Database::create(&*CONFIG_PATH)?;
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(TABLE)?;
        table.insert(key, value)?;
    }
    write_txn.commit()?;
    Ok(())
}
