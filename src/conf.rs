#![allow(dead_code, unused_variables)]

use anyhow::{anyhow, Result};
use home::home_dir;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

///! Handles application configuration.

/// The name of the default config directory expected
/// to be in the user's $HOME directory.
pub const CONFIG_DIR_NAME: &str = ".graphctl";

/// The name of the config file within the config directory.
pub const CONFIG_FILE_NAME: &str = "config.toml";

/// The name of the directory (within the config directory) where
/// where the database files (main db file, WAL, etc.) are stored.
pub const DB_DIR_NAME: &str = "data";

/// The name of the main database file.
pub const DB_FILE_NAME: &str = "graph.db";

/// Get the path to the app config directory.
pub fn get_config_dir(config_dir: Option<String>) -> Option<PathBuf> {
    // Was a config dir passed in?
    if let Some(cd) = config_dir {
        return Some(Path::new(&cd).into());
    };

    // Otherwise, use the default...
    let home = home_dir()?;
    let config_dir = home.join(CONFIG_DIR_NAME);
    Some(config_dir)
}

/// Given a config directory, get the path to the config file.
pub fn get_config_file(config_dir: &PathBuf) -> PathBuf {
    config_dir.join(CONFIG_FILE_NAME)
}

/// Given a config directory, get the path to the database directory.
pub fn get_db_dir(config_dir: &PathBuf) -> PathBuf {
    config_dir.join(DB_DIR_NAME)
}

/// Given a config directory, get the path to the database file.
pub fn get_db_file(config_dir: &PathBuf) -> PathBuf {
    config_dir.join(DB_DIR_NAME).join(DB_FILE_NAME)
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub conf_dir: PathBuf,

    pub db: DbConfig,
}

impl Config {
    pub fn new(config_dir: Option<String>) -> Result<Self> {
        let conf_dir = match get_config_dir(config_dir) {
            Some(cd) => cd,
            None => return Err(anyhow!("Could not get config directory.")),
        };
        Ok(Self {
            conf_dir,
            db: DbConfig {
                db_type: DBType::Local,
                remote_db_path: None,
                encrypt_replica: false,
            },
        })
    }

    pub fn read_from_file(config_dir: &PathBuf) -> Result<Self> {
        let conf_file = get_config_file(config_dir);
        let conf_str = std::fs::read_to_string(conf_file)?;
        let mut conf: Config = toml::from_str(&conf_str)?;
        conf.conf_dir = config_dir.clone();
        Ok(conf)
    }

    pub fn write_to_file(&self) -> Result<()> {
        let conf_file = get_config_file(&self.conf_dir);
        let conf_str = toml::to_string(self)?;
        std::fs::write(conf_file, conf_str)?;
        Ok(())
    }
}

/// Configuration for the underlying database.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DbConfig {
    /// The type/location of database to use.
    #[serde(rename = "type")]
    pub db_type: DBType,

    /// If `db_type` is `remote` or `remote-with-replica`,
    /// the path to the remote database.
    pub remote_db_path: Option<String>,

    /// If `db_type` is `local` or `remote-with-replica`,
    /// should the replica be encrypted?
    pub encrypt_replica: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub enum DBType {
    #[default]
    #[serde(rename = "local")]
    Local,

    #[serde(rename = "remote-only")]
    RemoteOnly,

    #[serde(rename = "remote-with-replica")]
    RemoteWithReplica,
}
