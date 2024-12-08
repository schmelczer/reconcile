use aide::OperationOutput;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use log::{info, warn};
use schemars::JsonSchema;
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SyncServerError {
    #[error("Initialisation error: {0}")]
    InitError(#[source] anyhow::Error),

    #[error("Client error: {0:?}")]
    ClientError(#[source] anyhow::Error),

    #[error("Server error: {0:?}")]
    ServerError(#[source] anyhow::Error),

    #[error("Not found: {0}")]
    NotFound(#[source] anyhow::Error),

    #[error("Unauthorized: {0}")]
    Unauthorized(#[source] anyhow::Error),

    #[error("Permission denied error: {0}")]
    PermissionDeniedError(#[source] anyhow::Error),
}

impl SyncServerError {
    pub fn serialize(&self) -> SerializedError {
        match self {
            Self::InitError(error) => format_anyhow_error(error),
            Self::ClientError(error) => format_anyhow_error(error),
            Self::ServerError(error) => format_anyhow_error(error),
            Self::NotFound(error) => format_anyhow_error(error),
            Self::Unauthorized(error) => format_anyhow_error(error),
            Self::PermissionDeniedError(error) => format_anyhow_error(error),
        }
    }
}

impl IntoResponse for SyncServerError {
    fn into_response(self) -> Response {
        let body = Json(self.serialize());

        match self {
            Self::InitError(_) => (StatusCode::INTERNAL_SERVER_ERROR, body).into_response(),
            Self::ClientError(_) => (StatusCode::BAD_REQUEST, body).into_response(),
            Self::ServerError(_) => (StatusCode::INTERNAL_SERVER_ERROR, body).into_response(),
            Self::NotFound(_) => (StatusCode::NOT_FOUND, body).into_response(),
            Self::Unauthorized(_) => (StatusCode::UNAUTHORIZED, body).into_response(),
            Self::PermissionDeniedError(_) => (StatusCode::FORBIDDEN, body).into_response(),
        }
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SerializedError {
    pub message: String,
    pub causes: Vec<String>,
}

fn format_anyhow_error(error: &anyhow::Error) -> SerializedError {
    let mut causes = vec![];
    let mut current_error = error.source();
    while let Some(error) = current_error {
        causes.push(error.to_string());
        current_error = error.source();
    }

    SerializedError {
        message: error.to_string(),
        causes,
    }
}

impl OperationOutput for SyncServerError {
    type Inner = Self;
}

pub fn init_error(error: anyhow::Error) -> SyncServerError {
    SyncServerError::InitError(error)
}

pub fn server_error(error: anyhow::Error) -> SyncServerError {
    warn!("Server error: {:?}", error);
    SyncServerError::ServerError(error)
}

pub fn client_error(error: anyhow::Error) -> SyncServerError {
    info!("Client error: {:?}", error);
    SyncServerError::ClientError(error)
}

pub fn not_found_error(error: anyhow::Error) -> SyncServerError {
    info!("Not found error: {:?}", error);
    SyncServerError::NotFound(error)
}

pub fn unauthorized_error(error: anyhow::Error) -> SyncServerError {
    info!("Unauthorized error: {:?}", error);
    SyncServerError::Unauthorized(error)
}

pub fn permission_denied_error(error: anyhow::Error) -> SyncServerError {
    info!("Permission denied error: {:?}", error);
    SyncServerError::PermissionDeniedError(error)
}
