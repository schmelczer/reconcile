use std::path::PathBuf;

use log::debug;
use serde::{Deserialize, Serialize};

use crate::consts::{DEFAULT_DATABASES_DIRECTORY_PATH, DEFAULT_MAX_CONNECTIONS_PER_VAULT};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    #[serde(default = "default_databases_directory_path")]
    pub databases_directory_path: PathBuf,

    #[serde(default = "default_max_connections_per_vault")]
    pub max_connections_per_vault: u32,
}

fn default_databases_directory_path() -> PathBuf {
    debug!("Using default databases directory path: {DEFAULT_DATABASES_DIRECTORY_PATH:?}");
    PathBuf::from(DEFAULT_DATABASES_DIRECTORY_PATH)
}

fn default_max_connections_per_vault() -> u32 {
    debug!("Using default max connections: {DEFAULT_MAX_CONNECTIONS_PER_VAULT}");
    DEFAULT_MAX_CONNECTIONS_PER_VAULT
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            databases_directory_path: default_databases_directory_path(),
            max_connections_per_vault: default_max_connections_per_vault(),
        }
    }
}
