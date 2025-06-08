use std::fmt::Display;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use log::{debug, error};
use serde::Serialize;
use thiserror::Error;
use ts_rs::TS;

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
    Unauthenticated(#[source] anyhow::Error),

    #[error("Permission denied error: {0}")]
    PermissionDeniedError(#[source] anyhow::Error),
}

impl SyncServerError {
    pub fn serialize(&self) -> SerializedError {
        match self {
            Self::InitError(error)
            | Self::ClientError(error)
            | Self::ServerError(error)
            | Self::NotFound(error)
            | Self::Unauthenticated(error)
            | Self::PermissionDeniedError(error) => error.into(),
        }
    }
}

#[derive(TS, Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SerializedError {
    pub error_type: &'static str,
    pub message: String,
    pub causes: Vec<String>,
}

impl Display for SerializedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.causes.is_empty() {
            write!(f, "\nCauses:\n")?;
            for cause in &self.causes {
                write!(f, "{}", &format!("- {cause}\n"))?;
            }
        }

        Ok(())
    }
}

impl IntoResponse for SyncServerError {
    fn into_response(self) -> Response {
        let body = Json(self.serialize());

        match self {
            Self::InitError(_) | Self::ServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
            }
            Self::ClientError(_) => (StatusCode::BAD_REQUEST, body).into_response(),
            Self::NotFound(_) => (StatusCode::NOT_FOUND, body).into_response(),
            Self::Unauthenticated(_) => (StatusCode::UNAUTHORIZED, body).into_response(),
            Self::PermissionDeniedError(_) => (StatusCode::FORBIDDEN, body).into_response(),
        }
    }
}

impl From<&anyhow::Error> for SerializedError {
    fn from(error: &anyhow::Error) -> SerializedError {
        let mut causes = vec![];
        let mut current_error = error.source();
        while let Some(error) = current_error {
            causes.push(error.to_string());
            current_error = error.source();
        }

        SerializedError {
            error_type: error.downcast_ref::<SyncServerError>().map_or(
                "UnknownError",
                |e| match e {
                    SyncServerError::InitError(_) => "InitError",
                    SyncServerError::ClientError(_) => "ClientError",
                    SyncServerError::ServerError(_) => "ServerError",
                    SyncServerError::NotFound(_) => "NotFound",
                    SyncServerError::Unauthenticated(_) => "Unauthenticated",
                    SyncServerError::PermissionDeniedError(_) => "PermissionDeniedError",
                },
            ),
            message: error.to_string(),
            causes,
        }
    }
}

pub fn init_error(error: anyhow::Error) -> SyncServerError {
    debug!("Initialization error: {error:?}");
    SyncServerError::InitError(error)
}

pub fn server_error(error: anyhow::Error) -> SyncServerError {
    debug!("Server error: {error:?}");
    SyncServerError::ServerError(error)
}

pub fn client_error(error: anyhow::Error) -> SyncServerError {
    debug!("Client error: {error:?}");
    SyncServerError::ClientError(error)
}

pub fn not_found_error(error: anyhow::Error) -> SyncServerError {
    debug!("Not found: {error:?}");
    SyncServerError::NotFound(error)
}

pub fn unauthenticated_error(error: anyhow::Error) -> SyncServerError {
    debug!("Unauthenticated user: {error:?}");
    SyncServerError::Unauthenticated(error)
}

pub fn permission_denied_error(error: anyhow::Error) -> SyncServerError {
    debug!("Permission denied: {error:?}");
    SyncServerError::PermissionDeniedError(error)
}
