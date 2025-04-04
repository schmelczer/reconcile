use log::debug;
use serde::{Deserialize, Serialize};

use crate::consts::{
    DEFAULT_HOST, DEFAULT_MAX_BODY_SIZE_MB, DEFAULT_MAX_CLIENTS_PER_VAULT, DEFAULT_PORT,
    DEFAULT_RESPONSE_TIMEOUT_SECONDS,
};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_max_body_size_mb")]
    pub max_body_size_mb: usize,

    #[serde(default = "default_max_clients_per_vault")]
    pub max_clients_per_vault: usize,

    #[serde(default = "default_response_timeout_seconds")]
    pub response_timeout_seconds: u64,
}

fn default_host() -> String {
    debug!("Using default server host: {DEFAULT_HOST}");
    DEFAULT_HOST.to_owned()
}

fn default_port() -> u16 {
    debug!("Using default server port: {DEFAULT_PORT}");
    DEFAULT_PORT
}

fn default_max_body_size_mb() -> usize {
    debug!("Using default max body size (MB): {DEFAULT_MAX_BODY_SIZE_MB}");
    DEFAULT_MAX_BODY_SIZE_MB
}

fn default_max_clients_per_vault() -> usize {
    debug!("Using default max clients per vault: {DEFAULT_MAX_CLIENTS_PER_VAULT}");
    DEFAULT_MAX_CLIENTS_PER_VAULT
}

fn default_response_timeout_seconds() -> u64 {
    debug!("Using default response timeout (seconds): {DEFAULT_RESPONSE_TIMEOUT_SECONDS}");
    DEFAULT_RESPONSE_TIMEOUT_SECONDS
}
