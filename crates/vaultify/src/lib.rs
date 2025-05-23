use std::{fs, path::Path, vec};

use anyhow::Result;
use directories::ProjectDirs;
use once_cell::sync::Lazy;
use redb::{Database, Error, TableDefinition};
use tempfile::{NamedTempFile, tempdir};
use tracing::{debug, info};

const APP_NAME: &str = "Anything";
const DB_FILE_NAME: &str = "anything.redb";
const TANTIVY_DIR_NAME: &str = "tantivy";
const TABLE_NAME: &str = "anything";

pub static VAULTIFY: Lazy<Vaultify> = Lazy::new(|| {
    #[cfg(feature = "mock")]
    {
        Vaultify::new_mock().expect("Failed to initialize mock Vaultify")
    }
    #[cfg(not(feature = "mock"))]
    {
        Vaultify::new().expect("Failed to initialize Vaultify")
    }
});

pub struct Vaultify {
    config_file: String,
    tantivy_path: String,
    db: Database,
    table_def: TableDefinition<'static, &'static str, String>,
}

impl Vaultify {
    fn new() -> Result<Self> {
        let (config_file, tantivy_path, _) = Self::get_directories()?;
        Self::setup(config_file, tantivy_path)
    }

    #[allow(dead_code)]
    fn new_mock() -> Result<Vaultify, anyhow::Error> {
        let config_file = NamedTempFile::new()
            .unwrap()
            .path()
            .to_str()
            .unwrap()
            .to_string();
        let tantivy_path = tempdir()?.path().to_str().unwrap().to_string();
        Self::setup(config_file, tantivy_path)
    }

    pub fn setup(config_file: String, tantivy_path: String) -> Result<Self> {
        if !Path::new(&tantivy_path).exists() {
            fs::create_dir_all(&tantivy_path)?;
        }

        let db: Database = Database::create(config_file.clone())?;
        let table_def: TableDefinition<'_, &'static str, String> = TableDefinition::new(TABLE_NAME);

        Ok(Vaultify {
            config_file,
            tantivy_path,
            db,
            table_def,
        })
    }

    fn get_directories() -> Result<(String, String, String)> {
        let proj_dirs = ProjectDirs::from("", "", APP_NAME)
            .ok_or_else(|| anyhow::anyhow!("Failed to get project directories"))?;

        let config_path = proj_dirs.config_dir().to_string_lossy().to_string();

        let config_file = proj_dirs
            .config_dir()
            .join(DB_FILE_NAME)
            .to_string_lossy()
            .to_string();

        let tantivy_path = proj_dirs
            .config_dir()
            .join(TANTIVY_DIR_NAME)
            .to_string_lossy()
            .to_string();

        Ok((config_file, tantivy_path, config_path))
    }

    pub fn init_vault() {
        if VAULTIFY.get("config_file").is_ok() {
            info!("Vault already initialized");
            return;
        }

        info!(
            "Vault not initialized, creating new vault in {}",
            VAULTIFY.config_file
        );

        Self::init_config().expect("Failed to initialize vault configuration");
    }

    fn init_config() -> Result<()> {
        VAULTIFY.set("config_file", VAULTIFY.config_file.clone())?;
        VAULTIFY.set("tantivy_path", VAULTIFY.tantivy_path.clone())?;
        VAULTIFY.set("indexed", "false".to_string())?;
        VAULTIFY.set("default_include_path", "/".to_string())?;
        VAULTIFY.set("default_exclude_path", serde_json::to_string(&vec![""])?)?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<String> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(self.table_def)?;

        let value = table
            .get(key)?
            .ok_or_else(|| Error::TableDoesNotExist(format!("Key '{}' not found", key)))?
            .value();

        Ok(value.to_string())
    }

    pub fn set(&self, key: &str, value: String) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(self.table_def)?;
            table.insert(key, value)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn cleanup(path: String) {
        if Path::new(&path).exists() {
            fs::remove_dir_all(&path).expect("Failed to remove: directory");
            debug!("Removed directory: {}", path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[cfg(not(feature = "mock"))]

    fn test_vaultify() {
        Vaultify::init_vault();
        let (_, _, config_path) = Vaultify::get_directories().expect("Failed to get directories");
        Vaultify::cleanup(config_path);
    }

    #[test]
    #[cfg(feature = "mock")]
    fn test_vaultify_mock() {
        Vaultify::init_vault();
    }
}
