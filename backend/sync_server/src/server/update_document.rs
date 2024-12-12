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

    let parent = state
        .database
        .get_document_version(&vault_id, &document_id, &request.parent_version_id, None)
        .await
        .map_err(server_error)?
        .map(Ok)
        .unwrap_or_else(|| {
            Err(not_found_error(anyhow!(
                "Parent version with id `{}` not found",
                &request.parent_version_id
            )))
        })?;

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

    if latest_version.is_deleted {
        return Err(client_error(anyhow!(
            "Document `{}` is deleted",
            document_id
        )));
    }

    let content_bytes = base64_to_bytes(&request.content_base64)
        .context("Failed to decode base64 content in request")
        .map_err(client_error)?;

    let merged_content = merge(&parent.content, &latest_version.content, &content_bytes)
        .context("Failed to decode bytes as UTF-8")
        .map_err(client_error)?;

    let new_version = StoredDocumentVersion {
        vault_id,
        document_id,
        version_id: latest_version.version_id + 1,
        content: merged_content,
        created_date: request.created_date,
        relative_path: request.relative_path,
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
