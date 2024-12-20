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
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), SyncServerError> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .context("Failed to initialise tracing")
        .map_err(init_error)?;

    let app_state = AppState::try_new()
        .await
        .context("Failed to initialise app state")
        .map_err(init_error)?;

    create_server(app_state)
        .await
        .context("Failed to start server")
        .map_err(init_error)
}
