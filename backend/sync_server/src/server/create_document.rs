use anyhow::Context;
use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use sync_lib::base64_to_bytes;

use super::{auth::auth, requests::CreateDocumentVersion};
use crate::{
    app_state::AppState,
    database::models::{DocumentVersionWithoutContent, StoredDocumentVersion, VaultId},
    errors::{client_error, server_error, SyncServerError},
};

#[axum::debug_handler]
pub async fn create_document(
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    Path(vault_id): Path<VaultId>,
    State(state): State<AppState>,
    Json(request): Json<CreateDocumentVersion>,
) -> Result<Json<DocumentVersionWithoutContent>, SyncServerError> {
    auth(&state, auth_header.token())?;

    let new_version = StoredDocumentVersion {
        vault_id,
        document_id: uuid::Uuid::new_v4(),
        version_id: 0,
        content: base64_to_bytes(&request.content_base64)
            .context("Cannot convert base64 encoded content to bytes")
            .map_err(client_error)?,
        created_date: request.created_date,
        relative_path: request.relative_path,
        updated_date: chrono::Utc::now(),
        is_binary: request.is_binary,
        is_deleted: false,
    };

    state
        .database
        .insert_document_version(&new_version, None)
        .await
        .map_err(server_error)?;

    Ok(Json(new_version.into()))
}
