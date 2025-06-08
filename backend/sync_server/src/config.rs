use std::path::Path;

use anyhow::{Context as _, Result};
use database_config::DatabaseConfig;
use log::info;
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
        let config = if path.exists() {
            info!(
                "Loading configuration from '{}'",
                path.canonicalize().unwrap().display()
            );
            Self::load_from_file(path).await?
        } else {
            Self::default()
        };

        config.write(path).await?;
        info!(
            "Updated configuration at '{}'",
            path.canonicalize().unwrap().display()
        );

        Ok(config)
    }

    pub async fn load_from_file(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path).await.with_context(|| {
            format!(
                "Cannot load configuration from disk from {}",
                path.display()
            )
        })?;

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
