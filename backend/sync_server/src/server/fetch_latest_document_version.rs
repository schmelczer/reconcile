use anyhow::anyhow;
use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use super::auth::auth;
use crate::{
    app_state::AppState,
    database::models::{DocumentId, DocumentVersion, VaultId},
    errors::{not_found_error, server_error, SyncServerError},
};

#[axum::debug_handler]
pub async fn fetch_latest_document_version(
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    Path((vault_id, document_id)): Path<(VaultId, DocumentId)>,
    State(state): State<AppState>,
) -> Result<Json<DocumentVersion>, SyncServerError> {
    auth(&state, auth_header.token())?;

    let latest_version = state
        .database
        .get_latest_document_version(&vault_id, &document_id, None)
        .await
        .map_err(server_error)?
        .map(Ok)
        .unwrap_or_else(|| {
            Err(not_found_error(anyhow!(
                "Latest document version of document `{}` not found",
                document_id
            )))
        })?;

    Ok(Json(latest_version.into()))
}
