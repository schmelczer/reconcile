use aide_axum_typed_multipart::TypedMultipart;
use anyhow::{Context as _, anyhow};
use axum::extract::{Path, State};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use axum_jsonschema::Json;
use log::info;
use schemars::JsonSchema;
use serde::Deserialize;
use sync_lib::{base64_to_bytes, is_file_type_mergable, merge};

use super::{
    app_state::AppState,
    auth::auth,
    requests::{UpdateDocumentVersion, UpdateDocumentVersionMultipart},
    responses::DocumentUpdateResponse,
};
use crate::{
    database::models::{DocumentId, StoredDocumentVersion, VaultId, VaultUpdateId},
    errors::{SyncServerError, client_error, not_found_error, server_error},
    utils::{deduped_file_paths, sanitize_path},
};

// This is required for aide to infer the path parameter types and names
#[derive(Deserialize, JsonSchema)]
pub struct PathParams {
    vault_id: VaultId,
    document_id: DocumentId,
}

#[axum::debug_handler]
pub async fn update_document_multipart(
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    Path(PathParams {
        vault_id,
        document_id,
    }): Path<PathParams>,
    State(state): State<AppState>,
    TypedMultipart(axum_typed_multipart::TypedMultipart(request)): TypedMultipart<
        UpdateDocumentVersionMultipart,
    >,
) -> Result<Json<DocumentUpdateResponse>, SyncServerError> {
    internal_update_document(
        auth_header,
        state,
        vault_id,
        document_id,
        request.parent_version_id,
        request.relative_path,
        request.content.contents.to_vec(),
    )
    .await
}

#[axum::debug_handler]
pub async fn update_document_json(
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    Path(PathParams {
        vault_id,
        document_id,
    }): Path<PathParams>,
    State(state): State<AppState>,
    Json(request): Json<UpdateDocumentVersion>,
) -> Result<Json<DocumentUpdateResponse>, SyncServerError> {
    let content_bytes = base64_to_bytes(&request.content_base64)
        .context("Failed to decode base64 content in request")
        .map_err(client_error)?;

    internal_update_document(
        auth_header,
        state,
        vault_id,
        document_id,
        request.parent_version_id,
        request.relative_path,
        content_bytes,
    )
    .await
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
async fn internal_update_document(
    auth_header: Authorization<Bearer>,
    mut state: AppState,
    vault_id: VaultId,
    document_id: DocumentId,
    parent_version_id: VaultUpdateId,
    relative_path: String,
    content: Vec<u8>,
) -> Result<Json<DocumentUpdateResponse>, SyncServerError> {
    auth(&state, auth_header.token())?;

    // No need for a transaction as document versions are immutable
    let parent_document = state
        .database
        .get_document_version(&vault_id, parent_version_id, None)
        .await
        .map_err(server_error)?
        .map_or_else(
            || {
                Err(not_found_error(anyhow!(
                    "Parent version with id `{}` not found",
                    parent_version_id
                )))
            },
            Ok,
        )?;

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

    if latest_version.is_deleted {
        transaction
            .rollback()
            .await
            .context("Failed to roll back transaction")
            .map_err(server_error)?;

        return Ok(Json(DocumentUpdateResponse::FastForwardUpdate(
            latest_version.into(),
        )));
    }

    let sanitized_relative_path = sanitize_path(&relative_path);

    // Return the latest version if the content and path are the same as the latest
    // version
    if content == latest_version.content && sanitized_relative_path == latest_version.relative_path
    {
        info!("Document content is the same as the latest version, skipping update");
        transaction
            .rollback()
            .await
            .context("Failed to roll back transaction")
            .map_err(server_error)?;

        return Ok(Json(DocumentUpdateResponse::FastForwardUpdate(
            latest_version.into(),
        )));
    }

    let merged_content = if is_file_type_mergable(&sanitized_relative_path) {
        merge(&parent_document.content, &latest_version.content, &content)
    } else {
        content.clone()
    };

    let is_different_from_request_content = merged_content != content;

    // We can only update the relative path if we're the first one to do so
    let new_relative_path = if parent_document.relative_path == latest_version.relative_path
        && latest_version.relative_path != sanitized_relative_path
    {
        let mut new_relative_path = String::default();
        for candidate in deduped_file_paths(&sanitized_relative_path) {
            if state
                .database
                .get_latest_document_by_path(&vault_id, &candidate, Some(&mut transaction))
                .await
                .map_err(server_error)?
                .is_none()
            {
                new_relative_path = candidate;
                break;
            }
        }

        new_relative_path
    } else {
        latest_version.relative_path.clone()
    };

    let new_version = StoredDocumentVersion {
        document_id,
        vault_update_id: last_update_id + 1,
        relative_path: new_relative_path,
        content: merged_content,
        updated_date: chrono::Utc::now(),
        is_deleted: false,
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

    Ok(Json(if is_different_from_request_content {
        DocumentUpdateResponse::MergingUpdate(new_version.into())
    } else {
        DocumentUpdateResponse::FastForwardUpdate(new_version.into())
    }))
}
