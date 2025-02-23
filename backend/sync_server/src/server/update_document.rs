use aide_axum_typed_multipart::TypedMultipart;
use anyhow::{Context as _, anyhow};
use axum::extract::{Path, State};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use axum_jsonschema::Json;
use chrono::{DateTime, Utc};
use log::info;
use regex::Regex;
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
    database::{
        self, Transaction,
        models::{DocumentId, StoredDocumentVersion, VaultId, VaultUpdateId},
    },
    errors::{SyncServerError, client_error, not_found_error, server_error},
    utils::sanitize_path,
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
        request.created_date,
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
        request.created_date,
        content_bytes,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn internal_update_document(
    auth_header: Authorization<Bearer>,
    state: AppState,
    vault_id: VaultId,
    document_id: DocumentId,
    parent_version_id: VaultUpdateId,
    relative_path: String,
    created_date: DateTime<Utc>,
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
    let new_relative_path = if parent_document.relative_path == latest_version.relative_path {
        get_deduped_file_name(
            &state.database,
            &vault_id,
            &mut transaction,
            &sanitized_relative_path,
        )
        .await?
    } else {
        latest_version.relative_path.clone()
    };

    let new_version = StoredDocumentVersion {
        vault_id,
        document_id,
        vault_update_id: last_update_id + 1,
        relative_path: new_relative_path,
        content: merged_content,
        created_date,
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

    Ok(Json(if is_different_from_request_content {
        DocumentUpdateResponse::MergingUpdate(new_version.into())
    } else {
        DocumentUpdateResponse::FastForwardUpdate(new_version.into())
    }))
}

// Only a single file can be on the same path, so we need to dedup the path
// in case the client is trying to rename the file to an existing file's name
// that it's unaware of.
async fn get_deduped_file_name(
    database: &database::Database,
    vault_id: &VaultId,
    transaction: &mut Transaction<'_>,
    path: &str,
) -> Result<String, SyncServerError> {
    let mut parts = path.rsplitn(2, '.');
    let (stem, extension) = match (parts.next(), parts.next()) {
        (Some(stem), maybe_extension) => (
            stem,
            maybe_extension
                .map(|ext| format!(".{ext}"))
                .unwrap_or_default(),
        ),
        _ => unreachable!("Path must have at least one part"),
    };

    let regex = Regex::new(r" \((\d+)\)$").unwrap();
    let start_number = regex
        .captures(stem)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse::<u32>().ok())
        .unwrap_or(0);

    let clean_stem = regex.replace(stem, "").to_string();

    for dedup_number in start_number.. {
        let proposed_path = if dedup_number == 0 {
            format!("{clean_stem}{extension}")
        } else {
            format!("{clean_stem} ({dedup_number}){extension}")
        };

        if database
            .get_latest_document_by_path(vault_id, &proposed_path, Some(transaction))
            .await
            .map_err(server_error)?
            .is_none()
        {
            return Ok(proposed_path.to_string());
        }
    }

    unreachable!("Loop must always return a value");
}
