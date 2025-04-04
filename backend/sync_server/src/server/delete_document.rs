use anyhow::Context as _;
use axum::extract::{Path, State};
use axum_jsonschema::Json;
use schemars::JsonSchema;
use serde::Deserialize;

use super::requests::DeleteDocumentVersion;
use crate::{
    app_state::{
        AppState,
        database::models::{
            DocumentId, DocumentVersionWithoutContent, StoredDocumentVersion, VaultId,
        },
    },
    errors::{SyncServerError, server_error},
    utils::sanitize_path,
};

// This is required for aide to infer the path parameter types and names
#[derive(Deserialize, JsonSchema)]
pub struct DeleteDocumentPathParams {
    vault_id: VaultId,
    document_id: DocumentId,
}

#[axum::debug_handler]
pub async fn delete_document(
    Path(DeleteDocumentPathParams {
        vault_id,
        document_id,
    }): Path<DeleteDocumentPathParams>,
    State(state): State<AppState>,
    Json(request): Json<DeleteDocumentVersion>,
) -> Result<Json<DocumentVersionWithoutContent>, SyncServerError> {
    let mut transaction = state
        .database
        .create_write_transaction(&vault_id)
        .await
        .map_err(server_error)?;

    let last_update_id = state
        .database
        .get_max_update_id_in_vault(&vault_id, Some(&mut transaction))
        .await
        .map_err(server_error)?;

    let new_version = StoredDocumentVersion {
        vault_update_id: last_update_id + 1,
        document_id,
        relative_path: sanitize_path(&request.relative_path),
        content: vec![],
        updated_date: chrono::Utc::now(),
        is_deleted: true,
    };

    state
        .database
        .insert_document_version(&vault_id, &new_version, Some(&mut transaction))
        .await
        .map_err(server_error)?;

    transaction
        .commit()
        .await
        .context("Failed to commit successful transaction")
        .map_err(server_error)?;

    state
        .broadcasts
        .send(vault_id, new_version.clone().into())
        .await;

    Ok(Json(new_version.into()))
}
