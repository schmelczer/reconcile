pub mod broadcasts;
pub mod database;

use std::ffi::OsString;

use anyhow::Result;
use broadcasts::Broadcasts;
use database::Database;

use crate::{config::Config, consts::DEFAULT_CONFIG_PATH};

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Config,
    pub database: Database,
    pub broadcasts: Broadcasts,
}

impl AppState {
    pub async fn try_new(config_path: Option<OsString>) -> Result<Self> {
        let config_path = config_path.unwrap_or_else(|| OsString::from(DEFAULT_CONFIG_PATH));
        let path = std::path::PathBuf::from(config_path);

        let config = Config::read_or_create(&path).await?;
        let database = Database::try_new(&config.database).await?;
        let broadcasts = Broadcasts::new(&config.server);

        Ok(Self {
            config,
            database,
            broadcasts,
        })
    }
}
