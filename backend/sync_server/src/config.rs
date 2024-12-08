use std::path::Path;

use anyhow::{Context, Result};
use database_config::DatabaseConfig;
use log::debug;
use serde::{Deserialize, Serialize};
use server_config::ServerConfig;
use tokio::fs;
use user_config::UserConfig;

pub mod database_config;
pub mod server_config;
pub mod user_config;

use crate::{
    consts::{DEFAULT_HOST, DEFAULT_MAX_CONNECTIONS, DEFAULT_PORT, DEFAULT_SQLITE_URL},
    errors::SyncServerError,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub users: UserConfig,
}

impl Config {
    pub async fn read(path: &Path) -> Result<Self> {
        Self::load_from_file(path)
            .await
            .with_context(|| format!("Cannot load configuration from disk from ({path:?})"))
    }

    pub async fn load_from_file(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .await
            .context("Failed to read configuration file")?;

        let config = serde_yaml::from_str(&contents).context("Failed to parse configuration")?;

        Ok(config)
    }

    pub async fn write(&self, path: &Path) -> Result<()> {
        let contents = serde_yaml::to_string(&self).context("Failed to serialize configuration")?;

        fs::write(path, contents)
            .await
            .context("Failed to write configuration to disk")
    }
}
