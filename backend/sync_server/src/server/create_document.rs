use anyhow::Context as _;
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

use super::{auth::auth, requests::CreateDocumentVersion, responses::DocumentUpdateResponse};
use crate::{
    app_state::AppState,
    database::models::{StoredDocumentVersion, VaultId},
    errors::{client_error, server_error, SyncServerError},
};

// This is required for aide to infer the path parameter types and names
#[derive(Deserialize, JsonSchema)]
pub struct PathParams {
    vault_id: VaultId,
}

/// Create a new document in case a document with the same doesn't exist
/// already. If a document with the same path exists, a new version is created
/// with their content merged.
#[axum::debug_handler]
pub async fn create_document(
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    Path(PathParams { vault_id }): Path<PathParams>,
    State(state): State<AppState>,
    Json(request): Json<CreateDocumentVersion>,
) -> Result<Json<DocumentUpdateResponse>, SyncServerError> {
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

    let maybe_existing_version = state
        .database
        .get_latest_document_by_path(&vault_id, &request.relative_path, Some(&mut transaction))
        .await
        .map_err(server_error)?
        .and_then(|doc| if doc.is_deleted { None } else { Some(doc) });

    let content_bytes = base64_to_bytes(&request.content_base64)
        .context("Failed to decode base64 content in request")
        .map_err(client_error)?;

    let response = if let Some(existing_version) = maybe_existing_version {
        if content_bytes == existing_version.content {
            info!(
                "Content of the new version is the same as the existing version. Not creating a \
                 new version."
            );

            transaction
                .rollback()
                .await
                .context("Failed to roll back unecceseary transaction")
                .map_err(server_error)?;

            return Ok(Json(DocumentUpdateResponse::FastForwardUpdate(
                existing_version.into(),
            )));
        }

        let merged_content = merge(
            &[], // the empty string is the first common parent of the two documents,
            &existing_version.content,
            &content_bytes,
        );

        let new_version = StoredDocumentVersion {
            vault_id,
            vault_update_id: last_update_id + 1,
            relative_path: request.relative_path,
            document_id: existing_version.document_id,
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

        DocumentUpdateResponse::MergingUpdate(new_version.into())
    } else {
        let new_version = StoredDocumentVersion {
            vault_id,
            vault_update_id: last_update_id + 1,
            document_id: uuid::Uuid::new_v4(),
            relative_path: request.relative_path,
            content: content_bytes,
            created_date: request.created_date,
            updated_date: chrono::Utc::now(),
            is_deleted: false,
        };

        state
            .database
            .insert_document_version(&new_version, Some(&mut transaction))
            .await
            .map_err(server_error)?;

        DocumentUpdateResponse::FastForwardUpdate(new_version.into())
    };

    transaction
        .commit()
        .await
        .context("Failed to commit successful transaction")
        .map_err(server_error)?;

    Ok(Json(response))
}
