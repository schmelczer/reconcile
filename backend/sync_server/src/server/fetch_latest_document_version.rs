use crate::app_state::AppState;
use crate::database::models::DocumentId;
use crate::database::models::DocumentVersion;
use crate::database::models::VaultId;
use crate::errors::not_found_error;
use crate::errors::server_error;
use crate::errors::SyncServerError;
use anyhow::anyhow;
use axum::extract::Path;
use axum::extract::State;
use axum::Json;

#[axum::debug_handler]
pub async fn fetch_latest_document_version(
    Path((vault_id, document_id)): Path<(VaultId, DocumentId)>,
    State(state): State<AppState>,
) -> Result<Json<DocumentVersion>, SyncServerError> {
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
