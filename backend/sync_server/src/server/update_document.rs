use anyhow::{anyhow, Context as _};
use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use log::info;
use schemars::JsonSchema;
use serde::Deserialize;
use sync_lib::{base64_to_bytes, merge};

use super::{auth::auth, requests::UpdateDocumentVersion};
use crate::{
    app_state::AppState,
    database::models::{DocumentId, DocumentVersion, StoredDocumentVersion, VaultId},
    errors::{client_error, not_found_error, server_error, SyncServerError},
};

// This is required for aide to infer the path parameter types and names
#[derive(Deserialize, JsonSchema)]
pub struct PathParams {
    vault_id: VaultId,
    document_id: DocumentId,
}

#[axum::debug_handler]
pub async fn update_document(
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    Path(PathParams {
        vault_id,
        document_id,
    }): Path<PathParams>,
    State(state): State<AppState>,
    Json(request): Json<UpdateDocumentVersion>,
) -> Result<Json<DocumentVersion>, SyncServerError> {
    auth(&state, auth_header.token())?;

    // No need for a transaction as document versions are immutable
    let parent_document = state
        .database
        .get_document_version(&vault_id, request.parent_version_id, None)
        .await
        .map_err(server_error)?
        .map_or_else(
            || {
                Err(not_found_error(anyhow!(
                    "Parent version with id `{}` not found",
                    request.parent_version_id
                )))
            },
            Ok,
        )?;

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

    let latest_version = state
        .database
        .get_latest_document(&vault_id, &document_id, Some(&mut transaction))
        .await
        .map_err(server_error)?
        .map_or_else(
            || {
                Err(not_found_error(anyhow!(
                    "Document with id `{document_id}` not found",
                )))
            },
            Ok,
        )?;

    let content_bytes = base64_to_bytes(&request.content_base64)
        .context("Failed to decode base64 content in request")
        .map_err(client_error)?;

    // Return the latest version if the content and path are the same as the latest
    // version
    if content_bytes == latest_version.content
        && request.relative_path == latest_version.relative_path
    {
        info!("Document content is the same as the latest version, skipping update");
        transaction
            .rollback()
            .await
            .context("Failed to roll back transaction")
            .map_err(server_error)?;

        return Ok(Json(latest_version.into()));
    }

    let merged_content = merge(
        &parent_document.content,
        &latest_version.content,
        &content_bytes,
    );

    // We can only update the relative path if we're the first one to do so
    let new_relative_path = if parent_document.relative_path == latest_version.relative_path {
        request.relative_path.clone()
    } else {
        latest_version.relative_path.clone()
    };

    let new_version = StoredDocumentVersion {
        vault_id,
        document_id,
        vault_update_id: last_update_id + 1,
        relative_path: new_relative_path,
        content: merged_content,
        created_date: request.created_date,
        updated_date: chrono::Utc::now(),
        is_deleted: latest_version.is_deleted,
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

    Ok(Json(new_version.into()))
}
