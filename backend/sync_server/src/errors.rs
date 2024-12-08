use aide::OperationOutput;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SyncServerError {
    #[error("Initialisation error: {0}")]
    InitError(#[source] anyhow::Error),

    #[error("Client error: {0}")]
    ClientError(#[source] anyhow::Error),

    #[error("Server error: {0}")]
    ServerError(#[source] anyhow::Error),

    #[error("Not found: {0}")]
    NotFound(#[source] anyhow::Error),

    #[error("Permission denier error: {0}")]
    PermissionDeniedError(#[source] anyhow::Error),
}

impl IntoResponse for SyncServerError {
    fn into_response(self) -> Response {
        let body = self.to_string();

        match self {
            Self::InitError(_) => (StatusCode::INTERNAL_SERVER_ERROR, body).into_response(),
            Self::ClientError(_) => (StatusCode::BAD_REQUEST, body).into_response(),
            Self::ServerError(_) => (StatusCode::INTERNAL_SERVER_ERROR, body).into_response(),
            Self::NotFound(_) => (StatusCode::NOT_FOUND, body).into_response(),
            Self::PermissionDeniedError(_) => (StatusCode::FORBIDDEN, body).into_response(),
        }
    }
}

impl OperationOutput for SyncServerError {
    type Inner = Self;
}

pub fn init_error(error: anyhow::Error) -> SyncServerError {
    SyncServerError::InitError(error)
}

pub fn server_error(error: anyhow::Error) -> SyncServerError {
    SyncServerError::ServerError(error)
}

pub fn client_error(error: anyhow::Error) -> SyncServerError {
    SyncServerError::ClientError(error)
}

pub fn not_found_error(error: anyhow::Error) -> SyncServerError {
    SyncServerError::NotFound(error)
}

pub fn permission_denied_error(error: anyhow::Error) -> SyncServerError {
    SyncServerError::PermissionDeniedError(error)
}
