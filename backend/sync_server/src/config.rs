use std::path::Path;

use anyhow::{Context, Result};
use database_config::DatabaseConfig;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use server_config::ServerConfig;
use tokio::fs;
use user_config::UserConfig;

pub mod database_config;
pub mod server_config;
pub mod user_config;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub users: UserConfig,
}

impl Config {
    pub async fn read_or_create(path: &Path) -> Result<Self> {
        if path.exists() {
            info!(
                "Loading configuration from {:?}",
                path.canonicalize().unwrap()
            );
            Self::load_from_file(path).await
        } else {
            let config = Config::default();
            config.write(path).await?;
            warn!(
                "Configuration file not found, wrote default configuration to {:?}",
                path.canonicalize().unwrap()
            );
            Ok(config)
        }
    }

    pub async fn load_from_file(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .await
            .with_context(|| format!("Cannot load configuration from disk from ({path:?})"))?;

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
