use std::sync::Arc;

use aide::{
    axum::{
        routing::{delete, get, post, put},
        ApiRouter,
    },
    openapi::{Info, OpenApi},
    scalar::Scalar,
    transform::TransformOpenApi,
};
use anyhow::{anyhow, Context as _, Result};
use app_state::AppState;
use axum::{
    extract::{DefaultBodyLimit, Request},
    http::{self, HeaderValue, Method},
    response::IntoResponse,
    Extension, Json,
};
use log::{error, info};
use tokio::signal;
use tower_http::{
    cors::CorsLayer,
    trace::{
        DefaultOnBodyChunk, DefaultOnEos, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse,
        TraceLayer,
    },
    LatencyUnit,
};
use tracing::{info_span, Level};

use crate::errors::{not_found_error, SerializedError};
mod app_state;
mod auth;
mod create_document;
mod delete_document;
mod fetch_latest_document_version;
mod fetch_latest_documents;
mod ping;
mod requests;
mod responses;
mod update_document;

pub async fn create_server() -> Result<()> {
    aide::gen::on_error(|err| error!("{err}"));
    aide::gen::extract_schemas(true);

    let app_state = AppState::try_new()
        .await
        .context("Failed to initialise app state")?;

    let address = format!(
        "{}:{}",
        &app_state.config.server.host, &app_state.config.server.port
    );

    let mut api = OpenApi {
        info: Info {
            title: "VaultLink sync server".to_owned(),
            summary: Some(
                "Simple API for syncing documents between concurrent clients.".to_owned(),
            ),
            description: Some(include_str!("../README.md").to_owned()),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            ..Info::default()
        },
        ..OpenApi::default()
    };

    let app = ApiRouter::new()
        .api_route("/ping", get(ping::ping))
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
        .route("/", Scalar::new("/api.json").axum_route())
        .route("/api.json", axum::routing::get(serve_api))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    info_span!(
                        "http_request",
                        method = ?request.method(),
                        uri = ?request.uri(),
                    )
                })
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                )
                .on_body_chunk(DefaultOnBodyChunk::new())
                .on_eos(DefaultOnEos::new())
                .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
        )
        .layer(DefaultBodyLimit::max(
            app_state.config.server.max_body_size_mb * 1024 * 1024,
        ))
        .layer(
            CorsLayer::new()
                .allow_origin("*".parse::<HeaderValue>().expect("Failed to parse origin"))
                .allow_headers([http::header::CONTENT_TYPE, http::header::AUTHORIZATION])
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE]),
        )
        .with_state(app_state)
        .finish_api_with(&mut api, api_docs)
        .layer(Extension(Arc::new(api))) // https://github.com/tamasfe/aide/blob/507f4a8822bc0c13cbda0f589da1e0f4cbcdb812/examples/example-axum/src/main.rs#L39
        .fallback(handler_404)
        .into_make_service();

    let listener = tokio::net::TcpListener::bind(address.clone())
        .await
        .with_context(|| format!("Failed to bind to address: {address}"))?;

    info!(
        "Listening on http://{}",
        listener
            .local_addr()
            .context("Failed to get local address")?
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .tcp_nodelay(true)
        .await
        .context("Failed to start server")
}

async fn serve_api(Extension(api): Extension<Arc<OpenApi>>) -> impl IntoResponse { Json(api) }

fn api_docs(api: TransformOpenApi<'_>) -> TransformOpenApi<'_> {
    api.default_response_with::<Json<SerializedError>, _>(|res| {
        res.example(SerializedError {
            message: "An error has occurred".to_owned(),
            causes: vec![],
        })
    })
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

async fn handler_404() -> impl IntoResponse { not_found_error(anyhow!("Page not found")) }
