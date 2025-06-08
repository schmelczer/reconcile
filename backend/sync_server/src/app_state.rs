pub mod cursors;
pub mod database;
pub mod websocket;

use std::ffi::OsString;

use anyhow::Result;
use cursors::Cursors;
use database::Database;
use websocket::broadcasts::Broadcasts;

use crate::{config::Config, consts::DEFAULT_CONFIG_PATH};

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Config,
    pub database: Database,
    pub cursors: Cursors,
    pub broadcasts: Broadcasts,
}

impl AppState {
    pub async fn try_new(config_path: Option<OsString>) -> Result<Self> {
        let config_path = config_path.unwrap_or_else(|| OsString::from(DEFAULT_CONFIG_PATH));
        let path = std::path::PathBuf::from(config_path);

        let config = Config::read_or_create(&path).await?;
        let broadcasts = Broadcasts::new(&config.server);
        let database = Database::try_new(&config.database, &broadcasts).await?;
        let cursors: Cursors = Cursors::new(&config.database, &broadcasts);

        Cursors::start_background_task(cursors.clone());

        Ok(Self {
            config,
            database,
            cursors,
            broadcasts,
        })
    }
}
