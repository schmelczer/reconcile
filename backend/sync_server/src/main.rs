mod app_state;
mod config;
mod consts;
mod database;
mod errors;
mod server;

use anyhow::{Context as _, Result};
use app_state::AppState;
use errors::{init_error, SyncServerError};
use server::create_server;

#[tokio::main]
async fn main() -> Result<(), SyncServerError> {
    tracing_subscriber::fmt::init();

    let app_state = AppState::try_new()
        .await
        .context("Failed to initialise app state")
        .map_err(init_error)?;

    create_server(app_state)
        .await
        .context("Failed to start server")
        .map_err(init_error)
}
