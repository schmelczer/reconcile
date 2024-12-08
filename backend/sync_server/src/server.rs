use crate::app_state::AppState;
use aide::{
    axum::{
        routing::{delete, get, post, put},
        ApiRouter,
    },
    openapi::{Info, OpenApi},
    scalar::Scalar,
};
use anyhow::Context;
use anyhow::Result;
use axum::response::{IntoResponse, Response};
use axum::{extract::DefaultBodyLimit, Extension};
use axum::{extract::WebSocketUpgrade, Json};
use log::info;
mod create_document;
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

    let mut api = OpenApi {
        info: Info {
            description: Some("an example API".to_string()),
            ..Info::default()
        },
        ..OpenApi::default()
    };

    let app = ApiRouter::new()
        .api_route(
            "/vaults/:vault_id/documents",
            get(fetch_latest_documents::fetch_latest_documents),
        )
        .api_route(
            "/vaults/:vault_id/documents",
            post(create_document::create_document),
        )
        .api_route(
            "/vaults/:vault_id/documents/:document_id",
            get(fetch_latest_document_version::fetch_latest_document_version),
        )
        .api_route(
            "/vaults/:vault_id/documents/:document_id",
            put(update_document::update_document),
        )
        .api_route(
            "/vaults/:vault_id/documents/:document_id",
            delete(delete_document::delete_document),
        )
        .api_route("/ws", get(handler))
        .route("/", Scalar::new("/api.json").axum_route())
        .route("/api.json", axum::routing::get(serve_api))
        .layer(DefaultBodyLimit::max(
            app_state.config.server.max_body_size_mb * 1024 * 1024,
        ))
        .with_state(app_state)
        .finish_api(&mut api)
        .layer(Extension(api))
        .into_make_service();

    let listener = tokio::net::TcpListener::bind(address.clone())
        .await
        .with_context(|| format!("Failed to bind to address: {}", address))?;

    info!(
        "Listening on http://{}",
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

async fn serve_api(Extension(api): Extension<OpenApi>) -> impl IntoResponse {
    Json(api)
}
