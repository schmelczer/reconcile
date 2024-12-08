use anyhow::{anyhow, Context};
use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use super::{auth::auth, requests::DeleteDocumentVersion};
use crate::{
    app_state::AppState,
    database::models::{DocumentId, StoredDocumentVersion, VaultId},
    errors::{not_found_error, server_error, SyncServerError},
};

#[axum::debug_handler]
pub async fn delete_document(
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    Path((vault_id, document_id)): Path<(VaultId, DocumentId)>,
    State(state): State<AppState>,
    Json(request): Json<DeleteDocumentVersion>,
) -> Result<(), SyncServerError> {
    auth(&state, auth_header.token())?;

    let mut transaction = state
        .database
        .create_transaction()
        .await
        .map_err(server_error)?;

    let latest_version = state
        .database
        .get_latest_document_version(&vault_id, &document_id, Some(&mut transaction))
        .await
        .map_err(server_error)?
        .map(Ok)
        .unwrap_or_else(|| {
            Err(not_found_error(anyhow!(
                "Latest document version of document `{}` not found",
                document_id
            )))
        })?;

    let new_version = StoredDocumentVersion {
        vault_id,
        document_id,
        version_id: latest_version.version_id + 1,
        content: vec![],
        created_date: request.created_date,
        updated_date: chrono::Utc::now(),
        relative_path: latest_version.relative_path,
        is_binary: latest_version.is_binary,
        is_deleted: true,
    };

    state
        .database
        .insert_document_version(&new_version, Some(&mut transaction))
        .await
        .map_err(server_error)?;

    transaction
        .commit()
        .await
        .context("Failed to commit successful transaction")
        .map_err(server_error)?;

    Ok(())
}
