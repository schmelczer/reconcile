use std::{path::PathBuf, time::Duration};

use log::debug;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::consts::{
    DEFAULT_CURSOR_TIMEOUT, DEFAULT_DATABASES_DIRECTORY_PATH, DEFAULT_MAX_CONNECTIONS_PER_VAULT,
};

#[serde_with::serde_as]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    #[serde(default = "default_databases_directory_path")]
    pub databases_directory_path: PathBuf,

    #[serde(default = "default_max_connections_per_vault")]
    pub max_connections_per_vault: u32,

    #[serde(default = "default_cursor_timeout", rename = "cursor_timeout_seconds")]
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    pub cursor_timeout: Duration,
}

fn default_databases_directory_path() -> PathBuf {
    debug!("Using default databases directory path: {DEFAULT_DATABASES_DIRECTORY_PATH:?}");
    PathBuf::from(DEFAULT_DATABASES_DIRECTORY_PATH)
}

fn default_max_connections_per_vault() -> u32 {
    debug!("Using default max connections: {DEFAULT_MAX_CONNECTIONS_PER_VAULT}");
    DEFAULT_MAX_CONNECTIONS_PER_VAULT
}

fn default_cursor_timeout() -> Duration {
    debug!("Using default cursor timeout: {DEFAULT_CURSOR_TIMEOUT:?}");
    DEFAULT_CURSOR_TIMEOUT
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            databases_directory_path: default_databases_directory_path(),
            max_connections_per_vault: default_max_connections_per_vault(),
            cursor_timeout: default_cursor_timeout(),
        }
    }
}
