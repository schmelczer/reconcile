use crate::app_state::AppState;
use anyhow::Context;
use anyhow::Result;
use axum::extract::DefaultBodyLimit;
use axum::extract::WebSocketUpgrade;
use axum::response::Response;
use axum::routing::delete;
use axum::{routing::get, routing::put, Router};
use log::info;

mod delete_document;
mod fetch_latest_document_version;
mod fetch_latest_documents;
mod requests;
mod update_document;

pub async fn create_server(app_state: AppState) -> Result<()> {
    let address = format!(
        "{}:{}",
        &app_state.config.server.host, &app_state.config.server.port
    );

    let app = Router::new()
        .route(
            "/vaults/:vault_id/documents/latest",
            get(fetch_latest_documents::fetch_latest_documents),
        )
        .route(
            "/vaults/:vault_id/documents/:document_id/versions/:parent_version_id",
            put(update_document::update_document),
        )
        .route(
            "/vaults/:vault_id/documents/:document_id",
            delete(delete_document::delete_document),
        )
        .route(
            "/vaults/:vault_id/documents/:document_id/versions/latest",
            get(fetch_latest_document_version::fetch_latest_document_version),
        )
        .route("/ws", get(handler))
        .layer(DefaultBodyLimit::max(
            app_state.config.server.max_body_size_mb * 1024 * 1024,
        ))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(address.clone())
        .await
        .with_context(|| format!("Failed to bind to address: {}", address))?;

    info!(
        "Listening on {}",
        listener
            .local_addr()
            .context("Failed to get local address")?
    );

    axum::serve(listener, app)
        .await
        .context("Failed to start server")
}

async fn handler(ws: WebSocketUpgrade) -> Response {
    ws.protocols(["graphql-ws", "graphql-transport-ws"])
        .on_upgrade(|socket| async {
            // ...
        })
}
