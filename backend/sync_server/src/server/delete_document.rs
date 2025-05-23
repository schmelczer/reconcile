use anyhow::Context as _;
use axum::{
    Extension,
    extract::{Path, State},
};
use axum_extra::TypedHeader;
use axum_jsonschema::Json;
use schemars::JsonSchema;
use serde::Deserialize;

use super::{device_id_header::DeviceIdHeader, requests::DeleteDocumentVersion};
use crate::{
    app_state::{
        AppState,
        broadcasts::VaultUpdate,
        database::models::{
            DocumentId, DocumentVersionWithoutContent, StoredDocumentVersion, VaultId,
        },
    },
    config::user_config::User,
    errors::{SyncServerError, server_error},
    utils::{normalize::normalize, sanitize_path::sanitize_path},
};

// This is required for aide to infer the path parameter types and names
#[derive(Deserialize, JsonSchema)]
pub struct DeleteDocumentPathParams {
    #[serde(deserialize_with = "normalize")]
    vault_id: VaultId,

    document_id: DocumentId,
}

#[axum::debug_handler]
pub async fn delete_document(
    Path(DeleteDocumentPathParams {
        vault_id,
        document_id,
    }): Path<DeleteDocumentPathParams>,
    Extension(user): Extension<User>,
    TypedHeader(user_agent): TypedHeader<DeviceIdHeader>,
    State(state): State<AppState>,
    Json(request): Json<DeleteDocumentVersion>,
) -> Result<Json<DocumentVersionWithoutContent>, SyncServerError> {
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

    let latest_content = state
        .database
        .get_latest_document(&vault_id, &document_id, Some(&mut transaction))
        .await
        .map_err(server_error)?
        .map_or_else(Vec::new, |version| version.content); // in case the document has never existed before deleting it

    let new_version = StoredDocumentVersion {
        vault_update_id: last_update_id + 1,
        document_id,
        relative_path: sanitize_path(&request.relative_path),
        content: latest_content, // copy the content from the latest version
        updated_date: chrono::Utc::now(),
        is_deleted: true,
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
                origin_device_id: request.device_id,
                document: new_version.clone().into(),
            },
        )
        .await;

    Ok(Json(new_version.into()))
}
