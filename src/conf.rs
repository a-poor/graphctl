///! Handles application configuration.

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use home::home_dir;


/// The name of the default config directory expected
/// to be in the user's $HOME directory.
const CONFIG_DIR_NAME: &str = ".graphctl";

/// The name of the config file within the config directory.
const CONFIG_FILE_NAME: &str = "config.toml";

/// The name of the directory (within the config directory) where
/// where the database files (main db file, WAL, etc.) are stored.
const DB_DIR_NAME: &str = "data";

/// The name of the main database file.
const DB_FILE_NAME: &str = "graph.db";

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

#[derive(Debug,Serialize,Deserialize)]
pub struct Config {
    db: DBConfig,
}

#[derive(Debug,Serialize,Deserialize)]
pub struct DBConfig {
    #[serde(rename = "type")] 
    db_type: DBType,
    remote_db_path: Option<String>,
    encrypt_replica: bool,
    read_your_writes: bool,
}

#[derive(Debug,Serialize,Deserialize,Default)]
pub enum DBType {
    #[default]
    #[serde(rename = "local")]
    Local,
    
    #[serde(rename = "remote-only")]
    RemoteOnly,

    #[serde(rename = "remote-with-replica")]
    RemoteWithReplica
}


