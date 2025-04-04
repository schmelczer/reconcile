mod auth;
mod create_document;
mod delete_document;
mod fetch_document_version;
mod fetch_document_version_content;
mod fetch_latest_document_version;
mod fetch_latest_documents;
mod ping;
mod requests;
mod responses;
mod update_document;
mod websocket;

use std::{ffi::OsString, sync::Arc, time::Duration};

use aide::{
    axum::{
        ApiRouter,
        routing::{delete, get, post, put},
    },
    openapi::{Info, OpenApi},
    scalar::Scalar,
    transform::TransformOpenApi,
};
use anyhow::{Context as _, Result, anyhow};
use auth::auth_middleware;
use axum::{
    Extension, Json,
    extract::{DefaultBodyLimit, Request},
    http::{self, HeaderValue, Method},
    middleware,
    response::IntoResponse,
    routing::IntoMakeService,
};
use log::{error, info};
use tokio::signal;
use tower_http::{
    LatencyUnit,
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer,
    trace::{
        DefaultOnBodyChunk, DefaultOnEos, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse,
        TraceLayer,
    },
};
use tracing::{Level, info_span};

use crate::{
    app_state::AppState,
    config::server_config::ServerConfig,
    errors::{SerializedError, client_error, not_found_error},
};

pub async fn create_server(config_path: Option<OsString>) -> Result<()> {
    aide::r#gen::on_error(|err| error!("{err}"));
    aide::r#gen::extract_schemas(true);

    let app_state = AppState::try_new(config_path)
        .await
        .context("Failed to initialise app state")?;

    let server_config = app_state.config.server.clone();

    let mut api = create_open_api();
    let app = ApiRouter::new()
        .nest("/", get_authed_routes(app_state.clone()))
        .api_route("/vaults/:vault_id/ping", get(ping::ping))
        .route("/vaults/:vault_id/ws", get(websocket::websocket_handler))
        .route("/", Scalar::new("/api.json").axum_route())
        .route("/api.json", axum::routing::get(serve_api))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(
            app_state.config.server.max_body_size_mb * 1024 * 1024,
        ))
        .layer(TimeoutLayer::new(Duration::from_secs(
            server_config.response_timeout_seconds,
        )))
        .layer(
            CorsLayer::new()
                .allow_origin("*".parse::<HeaderValue>().expect("Failed to parse origin"))
                .allow_headers([http::header::CONTENT_TYPE, http::header::AUTHORIZATION])
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE]),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    info_span!(
                        "http",
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
        .with_state(app_state)
        .finish_api_with(&mut api, add_api_docs_error_example)
        .layer(Extension(Arc::new(api))) // https://github.com/tamasfe/aide/blob/507f4a8822bc0c13cbda0f589da1e0f4cbcdb812/examples/example-axum/src/main.rs#L39
        .fallback(handle_404)
        .fallback(handle_405)
        .into_make_service();

    start_server(app, &server_config).await
}

async fn serve_api(Extension(api): Extension<Arc<OpenApi>>) -> impl IntoResponse { Json(api) }

fn create_open_api() -> OpenApi {
    OpenApi {
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
    }
}

fn add_api_docs_error_example(api: TransformOpenApi<'_>) -> TransformOpenApi<'_> {
    api.default_response_with::<Json<SerializedError>, _>(|res| {
        res.example(SerializedError {
            message: "An error has occurred".to_owned(),
            causes: vec![],
        })
    })
}

fn get_authed_routes(app_state: AppState) -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/vaults/:vault_id/documents",
            get(fetch_latest_documents::fetch_latest_documents),
        )
        .api_route(
            "/vaults/:vault_id/documents",
            post(create_document::create_document_multipart),
        )
        .api_route(
            "/vaults/:vault_id/documents/json",
            post(create_document::create_document_json),
        )
        .api_route(
            "/vaults/:vault_id/documents/:document_id",
            get(fetch_latest_document_version::fetch_latest_document_version),
        )
        .api_route(
            "/vaults/:vault_id/documents/:document_id",
            put(update_document::update_document_multipart),
        )
        .api_route(
            "/vaults/:vault_id/documents/:document_id/json",
            put(update_document::update_document_json),
        )
        .api_route(
            "/vaults/:vault_id/documents/:document_id/versions/:version_id",
            put(fetch_document_version::fetch_document_version),
        )
        .api_route(
            "/vaults/:vault_id/documents/:document_id/versions/:version_id/content",
            put(fetch_document_version_content::fetch_document_version_content),
        )
        .api_route(
            "/vaults/:vault_id/documents/:document_id",
            delete(delete_document::delete_document),
        )
        .layer(middleware::from_fn_with_state(app_state, auth_middleware))
}

async fn start_server(app: IntoMakeService<axum::Router>, config: &ServerConfig) -> Result<()> {
    let address = format!("{}:{}", config.host, config.port);
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
        () = ctrl_c => {},
        () = terminate => {},
    }
}

async fn handle_404() -> impl IntoResponse { not_found_error(anyhow!("Page not found")) }

async fn handle_405() -> impl IntoResponse { client_error(anyhow!("Method not allowed")) }
