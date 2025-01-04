use anyhow::Context as _;
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

use super::{app_state::AppState, auth::auth, requests::DeleteDocumentVersion};
use crate::{
    database::models::{DocumentId, StoredDocumentVersion, VaultId},
    errors::{server_error, SyncServerError},
    utils::sanitize_path,
};

// This is required for aide to infer the path parameter types and names
#[derive(Deserialize, JsonSchema)]
pub struct PathParams {
    vault_id: VaultId,
    document_id: DocumentId,
}

#[axum::debug_handler]
pub async fn delete_document(
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    Path(PathParams {
        vault_id,
        document_id,
    }): Path<PathParams>,
    State(state): State<AppState>,
    Json(request): Json<DeleteDocumentVersion>,
) -> Result<(), SyncServerError> {
    auth(&state, auth_header.token())?;

    let mut transaction = state
        .database
        .create_write_transaction()
        .await
        .map_err(server_error)?;

    let last_update_id = state
        .database
        .get_max_update_id_in_vault(&vault_id, Some(&mut transaction))
        .await
        .map_err(server_error)?;

    let new_version = StoredDocumentVersion {
        vault_id,
        vault_update_id: last_update_id + 1,
        document_id,
        relative_path: sanitize_path(&request.relative_path),
        content: vec![],
        created_date: request.created_date,
        updated_date: chrono::Utc::now(),
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
