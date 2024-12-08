use std::path::Path;

use anyhow::{Context, Result};
use log::debug;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{
    consts::{DEFAULT_HOST, DEFAULT_MAX_BODY_SIZE_MB, DEFAULT_PORT, DEFAULT_SQLITE_URL},
    errors::SyncServerError,
};
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
    DEFAULT_HOST.to_string()
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
