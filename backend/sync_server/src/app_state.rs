use crate::{config::Config, consts::CONFIG_PATH, database::Database};
use anyhow::Result;

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Config,
    pub database: Database,
}

impl AppState {
    pub async fn try_new() -> Result<Self> {
        let path = std::path::Path::new(CONFIG_PATH).canonicalize()?;

        let config = Config::read(&path).await?;
        let database = Database::try_new(&config.database).await?;

        Ok(Self { config, database })
    }
}
