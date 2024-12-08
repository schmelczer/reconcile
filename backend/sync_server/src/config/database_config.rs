use log::debug;
use serde::{Deserialize, Serialize};

use crate::consts::{DEFAULT_MAX_CONNECTIONS, DEFAULT_SQLITE_URL};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    #[serde(default = "default_sqlite_url")]
    pub sqlite_url: String,

    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

fn default_sqlite_url() -> String {
    debug!("Using default sqlite url: {}", DEFAULT_SQLITE_URL);
    DEFAULT_SQLITE_URL.to_string()
}

fn default_max_connections() -> u32 {
    debug!("Using default max connections: {}", DEFAULT_MAX_CONNECTIONS);
    DEFAULT_MAX_CONNECTIONS
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            sqlite_url: default_sqlite_url(),
            max_connections: default_max_connections(),
        }
    }
}
