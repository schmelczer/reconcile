use aide_axum_typed_multipart::TypedMultipart;
use anyhow::Context as _;
use axum::{
    Extension,
    extract::{Path, State},
};
use axum_extra::TypedHeader;
use axum_jsonschema::Json;
use schemars::JsonSchema;
use serde::Deserialize;
use sync_lib::base64_to_bytes;

use super::{
    device_id_header::DeviceIdHeader,
    requests::{CreateDocumentVersion, CreateDocumentVersionMultipart},
};
use crate::{
    app_state::{
        AppState,
        broadcasts::VaultUpdate,
        database::models::{
            DeviceId, DocumentId, DocumentVersionWithoutContent, StoredDocumentVersion, VaultId,
        },
    },
    config::user_config::User,
    errors::{SyncServerError, client_error, server_error},
    utils::{normalize::normalize, sanitize_path::sanitize_path},
};

// This is required for aide to infer the path parameter types and names
#[derive(Deserialize, JsonSchema)]
pub struct CreateDocumentPathParams {
    #[serde(deserialize_with = "normalize")]
    vault_id: VaultId,
}

/// Create a new document in case a document with the same doesn't exist
/// already. If a document with the same path exists, a new version is created
/// with their content merged.
#[axum::debug_handler]
pub async fn create_document_multipart(
    Path(CreateDocumentPathParams { vault_id }): Path<CreateDocumentPathParams>,
    Extension(user): Extension<User>,
    TypedHeader(user_agent): TypedHeader<DeviceIdHeader>,
    State(state): State<AppState>,
    TypedMultipart(axum_typed_multipart::TypedMultipart(request)): TypedMultipart<
        CreateDocumentVersionMultipart,
    >,
) -> Result<Json<DocumentVersionWithoutContent>, SyncServerError> {
    internal_create_document(
        user,
        user_agent,
        state,
        vault_id,
        request.document_id,
        request.relative_path,
        request.device_id,
        request.content.contents.to_vec(),
    )
    .await
}

/// Create a new document in case a document with the same doesn't exist
/// already. If a document with the same path exists, a new version is created
/// with their content merged.
#[axum::debug_handler]
pub async fn create_document_json(
    Path(CreateDocumentPathParams { vault_id }): Path<CreateDocumentPathParams>,
    Extension(user): Extension<User>,
    TypedHeader(user_agent): TypedHeader<DeviceIdHeader>,
    State(state): State<AppState>,
    Json(request): Json<CreateDocumentVersion>,
) -> Result<Json<DocumentVersionWithoutContent>, SyncServerError> {
    let content_bytes = base64_to_bytes(&request.content_base64)
        .context("Failed to decode base64 content in request")
        .map_err(client_error)?;

    internal_create_document(
        user,
        user_agent,
        state,
        vault_id,
        request.document_id,
        request.relative_path,
        request.device_id,
        content_bytes,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn internal_create_document(
    user: User,
    user_agent: DeviceIdHeader,
    state: AppState,
    vault_id: VaultId,
    document_id: Option<DocumentId>,
    relative_path: String,
    device_id: Option<DeviceId>,
    content: Vec<u8>,
) -> Result<Json<DocumentVersionWithoutContent>, SyncServerError> {
    let mut transaction = state
        .database
        .create_write_transaction(&vault_id)
        .await
        .map_err(server_error)?;

    let document_id = match document_id {
        Some(document_id) => {
            let existing_version = state
                .database
                .get_latest_document(&vault_id, &document_id, Some(&mut transaction))
                .await
                .map_err(server_error)?;

            if existing_version.is_some() {
                return Err(client_error(anyhow::anyhow!(
                    "Document with the same ID already exists"
                )));
            }

            document_id
        }
        None => uuid::Uuid::new_v4(),
    };

    let last_update_id = state
        .database
        .get_max_update_id_in_vault(&vault_id, Some(&mut transaction))
        .await
        .map_err(server_error)?;

    let sanitized_relative_path = sanitize_path(&relative_path);

    let new_version = StoredDocumentVersion {
        vault_update_id: last_update_id + 1,
        document_id,
        relative_path: sanitized_relative_path,
        content,
        updated_date: chrono::Utc::now(),
        is_deleted: false,
        user_id: user.name,
        device_id: user_agent.0,
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

    state
        .broadcasts
        .send(
            vault_id,
            VaultUpdate {
                origin_device_id: device_id,
                document: new_version.clone().into(),
            },
        )
        .await;

    Ok(Json(new_version.into()))
}
