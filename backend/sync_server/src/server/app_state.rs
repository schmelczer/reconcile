use anyhow::Result;

use crate::{config::Config, consts::CONFIG_PATH, database::Database};

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Config,
    pub database: Database,
}

impl AppState {
    pub async fn try_new() -> Result<Self> {
        let path = std::path::Path::new(CONFIG_PATH);

        let config = Config::read_or_create(path).await?;
        let database = Database::try_new(&config.database).await?;

        Ok(Self { config, database })
    }
}
