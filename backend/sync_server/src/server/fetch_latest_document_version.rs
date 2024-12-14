use anyhow::anyhow;
use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use schemars::JsonSchema;
use serde::Deserialize;

use super::auth::auth;
use crate::{
    app_state::AppState,
    database::models::{DocumentVersion, VaultId},
    errors::{not_found_error, server_error, SyncServerError},
};

// This is required for aide to infer the path parameter types and names
#[derive(Deserialize, JsonSchema)]
pub struct PathParams {
    vault_id: VaultId,
    relative_path: String,
}

#[axum::debug_handler]
pub async fn fetch_latest_document_version(
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    Path(PathParams {
        vault_id,
        relative_path,
    }): Path<PathParams>,
    State(state): State<AppState>,
) -> Result<Json<DocumentVersion>, SyncServerError> {
    auth(&state, auth_header.token())?;

    let latest_version = state
        .database
        .get_latest_document(&vault_id, &relative_path, None)
        .await
        .map_err(server_error)?
        .map(Ok)
        .unwrap_or_else(|| {
            Err(not_found_error(anyhow!(
                "Latest document version of document `{}` not found",
                relative_path
            )))
        })?;

    Ok(Json(latest_version.into()))
}
