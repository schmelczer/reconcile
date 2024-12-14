use anyhow::{anyhow, Context};
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
    database::models::{DocumentVersion, StoredDocumentVersion, VaultId},
    errors::{client_error, not_found_error, server_error, SyncServerError},
};

// This is required for aide to infer the path parameter types and names
#[derive(Deserialize, JsonSchema)]
pub struct PathParams {
    vault_id: VaultId,
    relative_path: String,
}

#[axum::debug_handler]
pub async fn update_document(
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    Path(PathParams {
        vault_id,
        relative_path,
    }): Path<PathParams>,
    State(state): State<AppState>,
    Json(request): Json<UpdateDocumentVersion>,
) -> Result<Json<DocumentVersion>, SyncServerError> {
    auth(&state, auth_header.token())?;

    let parent_content = if let Some(parent_version_id) = request.parent_version_id {
        state
            .database
            .get_document_version(&vault_id, parent_version_id, None)
            .await
            .map_err(server_error)?
            .map(Ok)
            .unwrap_or_else(|| {
                Err(not_found_error(anyhow!(
                    "Parent version with id `{}` not found",
                    parent_version_id
                )))
            })
            .map(|version| version.content)
    } else {
        // the empty string is the first common parent of the two documents
        Ok(Vec::default())
    }?;

    let content_bytes = base64_to_bytes(&request.content_base64)
        .context("Failed to decode base64 content in request")
        .map_err(client_error)?;

    let mut transaction = state
        .database
        .create_transaction()
        .await
        .map_err(server_error)?;

    let mut last_update_id = state
        .database
        .get_max_update_id_in_vault(&vault_id, Some(&mut transaction))
        .await
        .map_err(server_error)?;

    let latest_version = state
        .database
        .get_latest_document(&vault_id, &relative_path, Some(&mut transaction))
        .await
        .map_err(server_error)?;

    if let Some(latest) = latest_version.as_ref() {
        if content_bytes == latest.content && relative_path == latest.relative_path {
            info!("Document content is the same as the latest version, skipping update");
            transaction
                .rollback()
                .await
                .context("Failed to rollback transaction")
                .map_err(server_error)?;

            return Ok(Json(latest.clone().into()));
        } else if relative_path != latest.relative_path {
            let delete_at_previous_path = StoredDocumentVersion {
                vault_id: vault_id.clone(),
                vault_update_id: last_update_id + 1,
                relative_path: latest.relative_path.clone(),
                content: vec![],
                created_date: request.created_date,
                updated_date: chrono::Utc::now(),
                is_deleted: true,
            };

            last_update_id += 1;

            state
                .database
                .insert_document_version(&delete_at_previous_path, Some(&mut transaction))
                .await
                .map_err(server_error)?;
        }
    }

    let latest_version_content = latest_version
        .map(|v| v.content)
        .unwrap_or_else(Vec::default);

    let merged_content = merge(&parent_content, &latest_version_content, &content_bytes)
        .context("Failed to decode bytes as UTF-8")
        .map_err(client_error)?;

    let new_version = StoredDocumentVersion {
        vault_id,
        vault_update_id: last_update_id + 1,
        relative_path,
        content: merged_content,
        created_date: request.created_date,
        updated_date: chrono::Utc::now(),
        is_deleted: false,
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
