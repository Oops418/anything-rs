use std::{fs, path::Path, vec};

use anyhow::Result;
use directories::{ProjectDirs, UserDirs};
use once_cell::sync::Lazy;
use redb::{Database, Error, ReadableTable, TableDefinition};
#[cfg(feature = "mock")]
use tempfile::{NamedTempFile, tempdir};
use tracing::{debug, info};

const APP_NAME: &str = "Anything";
const DB_FILE_NAME: &str = "anything.redb";
const TANTIVY_DIR_NAME: &str = "tantivy";
const TABLE_NAME: &str = "anything";

pub static VAULTIFY: Lazy<Vaultify> = Lazy::new(|| {
    #[cfg(feature = "mock")]
    {
        Vaultify::new().expect("Failed to initialize mock Vaultify")
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
    #[cfg(not(feature = "mock"))]
    fn new() -> Result<Self> {
        let (config_file, tantivy_path, _) = Self::get_directories()?;
        Self::setup(config_file, tantivy_path)
    }

    #[cfg(feature = "mock")]
    fn new() -> Result<Self> {
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

    #[cfg(not(feature = "mock"))]
    fn init_config() -> Result<()> {
        let user_dirs =
            UserDirs::new().ok_or_else(|| anyhow::anyhow!("Failed to get user directories"))?;
        let home_dir = user_dirs.home_dir().to_string_lossy().to_string();
        let music_dir = user_dirs
            .audio_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get audio directory"))?
            .to_str()
            .expect("Failed to convert path to string with music_dir");
        let picture_dir = user_dirs
            .picture_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get picture directory"))?
            .to_str()
            .expect("Failed to convert path to string with picture_dir");
        let (_, _, config_path) = Vaultify::get_directories().expect("Failed to get directories");
        VAULTIFY.set("home_dir", home_dir)?;
        VAULTIFY.set("config_file", VAULTIFY.config_file.clone())?;
        VAULTIFY.set("tantivy_path", VAULTIFY.tantivy_path.clone())?;
        VAULTIFY.set("indexed", "false".to_string())?;
        VAULTIFY.set("refresh", "false".to_string())?;
        VAULTIFY.set("default_include_path", "/".to_string())?;
        VAULTIFY.set("indexed_files", "0".to_string())?;
        VAULTIFY.set("indexed_progress", "0.0".to_string())?;
        VAULTIFY.set("version", env!("CARGO_PKG_VERSION").to_string())?;
        VAULTIFY.set(
            "default_exclude_path",
            serde_json::to_string(&vec![
                "/System",
                "/bin",
                "/dev",
                "/sbin",
                "/lib",
                "/private",
                "/.VolumeIcon.icns",
                music_dir,
                picture_dir,
                config_path.as_str(),
            ])?,
        )?;
        Ok(())
    }

    #[cfg(feature = "mock")]
    fn init_config() -> Result<()> {
        let user_dirs =
            UserDirs::new().ok_or_else(|| anyhow::anyhow!("Failed to get user directories"))?;
        let home_dir = user_dirs.home_dir().to_string_lossy().to_string();
        VAULTIFY.set("home_dir", home_dir)?;
        VAULTIFY.set("config_file", VAULTIFY.config_file.clone())?;
        VAULTIFY.set("tantivy_path", VAULTIFY.tantivy_path.clone())?;
        VAULTIFY.set("indexed", "false".to_string())?;
        VAULTIFY.set("refresh", "false".to_string())?;
        VAULTIFY.set("default_include_path", "/".to_string())?;
        VAULTIFY.set("indexed_files", "0".to_string())?;
        VAULTIFY.set("indexed_progress", "0.0".to_string())?;
        VAULTIFY.set("version", env!("CARGO_PKG_VERSION").to_string())?;
        VAULTIFY.set(
            "default_exclude_path",
            serde_json::to_string(&vec!["None"])?,
        )?;
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

    pub fn list_all() -> Result<Vec<(String, String)>> {
        let read_txn: redb::ReadTransaction = VAULTIFY.db.begin_read()?;
        let table = read_txn.open_table(VAULTIFY.table_def)?;

        let mut entries = Vec::new();
        let mut iter = table.iter()?;
        while let Some(entry) = iter.next() {
            let (key, value) = entry?;
            entries.push((key.value().to_string(), value.value().to_string()));
        }
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(feature = "mock"))]
    #[ignore]
    fn clean_vaultify() {
        Vaultify::init_vault();
        let (_, _, config_path) = Vaultify::get_directories().expect("Failed to get directories");
        Vaultify::cleanup(config_path);
    }

    #[test]
    #[cfg(feature = "mock")]
    fn test_vaultify() {
        Vaultify::init_vault();
    }
}
