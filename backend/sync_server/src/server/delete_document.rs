use anyhow::{anyhow, Context};
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

use super::{auth::auth, requests::DeleteDocumentVersion};
use crate::{
    app_state::AppState,
    database::models::{StoredDocumentVersion, VaultId},
    errors::{not_found_error, server_error, SyncServerError},
};

// This is required for aide to infer the path parameter types and names
#[derive(Deserialize, JsonSchema)]
pub struct PathParams {
    vault_id: VaultId,
    relative_path: String,
}

#[axum::debug_handler]
pub async fn delete_document(
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    Path(PathParams {
        vault_id,
        relative_path,
    }): Path<PathParams>,
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
        .get_latest_document(&vault_id, &relative_path, Some(&mut transaction))
        .await
        .map_err(server_error)?
        .map(Ok)
        .unwrap_or_else(|| {
            Err(not_found_error(anyhow!(
                "Latest document version of document `{}` not found",
                relative_path
            )))
        })?;

    let new_version = StoredDocumentVersion {
        vault_id,
        relative_path,
        version_id: latest_version.version_id + 1,
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
