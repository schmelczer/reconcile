use axum::Json;

use crate::{database::models::PingResponse, errors::SyncServerError};

#[axum::debug_handler]
pub async fn ping() -> Result<Json<PingResponse>, SyncServerError> {
    Ok(Json(PingResponse {
        server_version: env!("CARGO_PKG_VERSION").to_string(),
    }))
}
