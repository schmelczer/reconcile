use std::path::PathBuf;

use log::debug;
use serde::{Deserialize, Serialize};

use crate::consts::{DEFAULT_DATABASES_DIRECTORY_PATH, DEFAULT_MAX_CONNECTIONS};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    #[serde(default = "default_databases_directory_path")]
    pub databases_directory_path: PathBuf,

    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

fn default_databases_directory_path() -> PathBuf {
    debug!("Using default databases directory path: {DEFAULT_DATABASES_DIRECTORY_PATH:?}");
    PathBuf::from(DEFAULT_DATABASES_DIRECTORY_PATH)
}

fn default_max_connections() -> u32 {
    debug!("Using default max connections: {DEFAULT_MAX_CONNECTIONS}");
    DEFAULT_MAX_CONNECTIONS
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            databases_directory_path: default_databases_directory_path(),
            max_connections: default_max_connections(),
        }
    }
}
