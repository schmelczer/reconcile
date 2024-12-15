use anyhow::Context;
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
use sync_lib::{base64_to_bytes, merge};

use super::{auth::auth, requests::CreateDocumentVersion};
use crate::{
    app_state::AppState,
    database::models::{DocumentVersion, StoredDocumentVersion, VaultId},
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
) -> Result<Json<DocumentVersion>, SyncServerError> {
    auth(&state, auth_header.token())?;

    let mut transaction = state
        .database
        .create_transaction()
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

    let new_version = if let Some(existing_version) = maybe_existing_version {
        let content_bytes = base64_to_bytes(&request.content_base64)
            .context("Failed to decode base64 content in request")
            .map_err(client_error)?;

        let merged_content = merge(
            &[], // the empty string is the first common parent of the two documents,
            &existing_version.content,
            &content_bytes,
        )
        .context("Failed to decode bytes as UTF-8")
        .map_err(client_error)?;

        StoredDocumentVersion {
            vault_id,
            vault_update_id: last_update_id + 1,
            relative_path: request.relative_path,
            document_id: existing_version.document_id,
            content: merged_content,
            created_date: request.created_date,
            updated_date: chrono::Utc::now(),
            is_deleted: false,
        }
    } else {
        StoredDocumentVersion {
            vault_id,
            vault_update_id: last_update_id + 1,
            document_id: uuid::Uuid::new_v4(),
            relative_path: request.relative_path,
            content: base64_to_bytes(&request.content_base64)
                .context("Cannot convert base64 encoded content to bytes")
                .map_err(client_error)?,
            created_date: request.created_date,
            updated_date: chrono::Utc::now(),
            is_deleted: false,
        }
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
