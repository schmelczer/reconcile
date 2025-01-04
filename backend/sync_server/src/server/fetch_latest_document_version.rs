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

use super::{app_state::AppState, auth::auth};
use crate::{
    database::models::{DocumentId, DocumentVersion, VaultId},
    errors::{not_found_error, server_error, SyncServerError},
};

// This is required for aide to infer the path parameter types and names
#[derive(Deserialize, JsonSchema)]
pub struct PathParams {
    vault_id: VaultId,
    document_id: DocumentId,
}

#[axum::debug_handler]
pub async fn fetch_latest_document_version(
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    Path(PathParams {
        vault_id,
        document_id,
    }): Path<PathParams>,
    State(state): State<AppState>,
) -> Result<Json<DocumentVersion>, SyncServerError> {
    auth(&state, auth_header.token())?;

    let latest_version = state
        .database
        .get_latest_document(&vault_id, &document_id, None)
        .await
        .map_err(server_error)?
        .map(Ok)
        .unwrap_or_else(|| {
            Err(not_found_error(anyhow!(
                "Document with id `{document_id}` not found",
            )))
        })?;

    Ok(Json(latest_version.into()))
}
