use aide_axum_typed_multipart::TypedMultipart;
use anyhow::Context as _;
use axum::extract::{Path, State};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use axum_jsonschema::Json;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::Deserialize;
use sync_lib::base64_to_bytes;

use super::{
    app_state::AppState,
    auth::auth,
    requests::{CreateDocumentVersion, CreateDocumentVersionMultipart},
};
use crate::{
    database::models::{DocumentVersionWithoutContent, StoredDocumentVersion, VaultId},
    errors::{SyncServerError, client_error, server_error},
    utils::sanitize_path,
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
pub async fn create_document_multipart(
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    Path(PathParams { vault_id }): Path<PathParams>,
    State(state): State<AppState>,
    TypedMultipart(axum_typed_multipart::TypedMultipart(request)): TypedMultipart<
        CreateDocumentVersionMultipart,
    >,
) -> Result<Json<DocumentVersionWithoutContent>, SyncServerError> {
    internal_create_document(
        auth_header,
        state,
        vault_id,
        request.relative_path,
        request.created_date,
        request.content.contents.to_vec(),
    )
    .await
}

/// Create a new document in case a document with the same doesn't exist
/// already. If a document with the same path exists, a new version is created
/// with their content merged.
#[axum::debug_handler]
pub async fn create_document_json(
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    Path(PathParams { vault_id }): Path<PathParams>,
    State(state): State<AppState>,
    Json(request): Json<CreateDocumentVersion>,
) -> Result<Json<DocumentVersionWithoutContent>, SyncServerError> {
    let content_bytes = base64_to_bytes(&request.content_base64)
        .context("Failed to decode base64 content in request")
        .map_err(client_error)?;

    internal_create_document(
        auth_header,
        state,
        vault_id,
        request.relative_path,
        request.created_date,
        content_bytes,
    )
    .await
}

async fn internal_create_document(
    auth_header: Authorization<Bearer>,
    state: AppState,
    vault_id: VaultId,
    relative_path: String,
    created_date: DateTime<Utc>,
    content: Vec<u8>,
) -> Result<Json<DocumentVersionWithoutContent>, SyncServerError> {
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

    let sanitized_relative_path = sanitize_path(&relative_path);

    let new_version = StoredDocumentVersion {
        vault_id,
        vault_update_id: last_update_id + 1,
        document_id: uuid::Uuid::new_v4(),
        relative_path: sanitized_relative_path,
        content,
        created_date,
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
