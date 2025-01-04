mod config;
mod consts;
mod database;
mod errors;
mod server;
mod utils;

use anyhow::{Context as _, Result};
use errors::{init_error, SyncServerError};
use log::info;
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

    info!(
        "Starting VaultLink server version {}",
        env!("CARGO_PKG_VERSION")
    );

    create_server()
        .await
        .context("Failed to start server")
        .map_err(init_error)
}
