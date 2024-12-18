use log::debug;
use serde::{Deserialize, Serialize};

use crate::consts::{DEFAULT_HOST, DEFAULT_MAX_BODY_SIZE_MB, DEFAULT_PORT};
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_max_body_size_mb")]
    pub max_body_size_mb: usize,
}

fn default_host() -> String {
    debug!("Using default server host: {}", DEFAULT_HOST);
    DEFAULT_HOST.to_owned()
}

fn default_port() -> u16 {
    debug!("Using default server port: {}", DEFAULT_PORT);
    DEFAULT_PORT
}

fn default_max_body_size_mb() -> usize {
    debug!(
        "Using default max body size (MB): {}",
        DEFAULT_MAX_BODY_SIZE_MB
    );
    DEFAULT_MAX_BODY_SIZE_MB
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            max_body_size_mb: default_max_body_size_mb(),
        }
    }
}
