mod app_state;
mod cli;
mod config;
mod consts;
mod errors;
mod server;
mod utils;

use std::process::ExitCode;

use anyhow::{Context as _, Result};
use clap::Parser;
use cli::args::Args;
use errors::{SyncServerError, init_error};
use log::info;
use server::create_server;
use tracing_subscriber::{EnvFilter, fmt::format, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();

    let mut result = set_up_logging(&args);

    if result.is_ok() {
        result = start_server(args).await;
    }

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}", e.serialize().to_string());
            ExitCode::FAILURE
        }
    }
}

fn set_up_logging(args: &Args) -> Result<(), SyncServerError> {
    let level_filter = match args.verbose.log_level_filter() {
        // We don't want to allow disabling all logging
        log::LevelFilter::Off | log::LevelFilter::Error => tracing::Level::ERROR,
        log::LevelFilter::Warn => tracing::Level::WARN,
        log::LevelFilter::Info => tracing::Level::INFO,
        log::LevelFilter::Debug => tracing::Level::DEBUG,
        log::LevelFilter::Trace => tracing::Level::TRACE,
    };

    let env_filter = EnvFilter::builder()
        .with_default_directive(level_filter.into())
        .from_env()
        .context("Failed to create logging env filter")
        .map_err(init_error)?;

    let use_colors = args.color.use_colors();

    let is_debug_mode = args.verbose.log_level_filter() >= log::LevelFilter::Debug;

    tracing_subscriber::fmt()
        .with_ansi(use_colors)
        .with_env_filter(env_filter)
        .event_format(
            format()
                .without_time()
                .with_target(is_debug_mode)
                .with_line_number(is_debug_mode)
                .compact(),
        )
        .finish()
        .try_init()
        .context("Failed to initialise tracing")
        .map_err(init_error)?;

    Ok(())
}

async fn start_server(args: Args) -> Result<(), SyncServerError> {
    info!(
        "Starting VaultLink server version {}",
        env!("CARGO_PKG_VERSION")
    );

    create_server(args.config_path)
        .await
        .context("Failed to start server")
        .map_err(init_error)
}
