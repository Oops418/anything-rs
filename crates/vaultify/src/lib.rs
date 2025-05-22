use std::{fs, path::Path};

use once_cell::sync::Lazy;
use redb::{Database, Error, ReadTransaction, TableDefinition, WriteTransaction};
use tracing::info;

pub static VAULTIFY: Lazy<Vaultify> = Lazy::new(|| {
    let config_path: String =
        "/Users/kxyang/Personal/CodeSpaces/anything-rs/anything.redb".to_string();
    let tantivy_path = "/Users/kxyang/Personal/CodeSpaces/anything-rs/tantivy-test".to_string();

    if !Path::new(&tantivy_path).exists() {
        fs::create_dir_all(&tantivy_path).expect("Failed to create tantivy directory");
    }

    let db = Database::create(&config_path).expect("Failed to open database");
    let table_def = TableDefinition::new("anything");

    Vaultify {
        config_path,
        tantivy_path,
        db,
        table_def,
    }
});

pub struct Vaultify {
    config_path: String,
    tantivy_path: String,
    db: Database,
    table_def: TableDefinition<'static, &'static str, String>,
}

impl Vaultify {
    pub fn init_vault() {
        match VAULTIFY.get("config_path") {
            Ok(_config_path) => {
                info!("vault initialized: {}", VAULTIFY.config_path);
                return;
            }
            Err(_e) => {
                info!(
                    "vault not initialized, creating new vault in {}",
                    VAULTIFY.config_path
                );
                VAULTIFY
                    .set("config_path", String::from(VAULTIFY.config_path.clone()))
                    .expect("Failed to set config_path");
                VAULTIFY
                    .set("tantivy_path", String::from(VAULTIFY.tantivy_path.clone()))
                    .expect("Failed to set tantivy_path");
                VAULTIFY
                    .set("indexed", String::from("false"))
                    .expect("Failed to set indexed");
            }
        }
    }

    pub fn get(&self, key: &str) -> Result<String, Error> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(self.table_def)?;
        let value = table.get(key)?.unwrap().value();
        Ok(String::from(value))
    }

    pub fn set(&self, key: &str, value: String) -> Result<(), Error> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(self.table_def)?;
            table.insert(key, value)?;
        }
        write_txn.commit()?;
        Ok(())
    }
}
