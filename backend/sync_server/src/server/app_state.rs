use std::ffi::OsString;

use anyhow::Result;

use crate::{config::Config, consts::DEFAULT_CONFIG_PATH, database::Database};

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Config,
    pub database: Database,
}

impl AppState {
    pub async fn try_new(config_path: Option<OsString>) -> Result<Self> {
        let config_path = config_path.unwrap_or_else(|| OsString::from(DEFAULT_CONFIG_PATH));
        let path = std::path::PathBuf::from(config_path);

        let config = Config::read_or_create(&path).await?;
        let database = Database::try_new(&config.database).await?;

        Ok(Self { config, database })
    }
}
