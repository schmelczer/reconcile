use anyhow::Context as _;
use axum::{
    Extension, Json,
    extract::{Path, State},
};
use axum_extra::TypedHeader;
use axum_typed_multipart::TypedMultipart;
use serde::Deserialize;

use super::{device_id_header::DeviceIdHeader, requests::CreateDocumentVersion};
use crate::{
    app_state::{
        AppState,
        database::models::{DocumentVersionWithoutContent, StoredDocumentVersion, VaultId},
    },
    config::user_config::User,
    errors::{SyncServerError, client_error, server_error},
    utils::{normalize::normalize, sanitize_path::sanitize_path},
};

#[derive(Deserialize)]
pub struct CreateDocumentPathParams {
    #[serde(deserialize_with = "normalize")]
    vault_id: VaultId,
}

/// Create a new document in case a document with the same doesn't exist
/// already. If a document with the same path exists, a new version is created
/// with their content merged.
#[axum::debug_handler]
pub async fn create_document(
    Path(CreateDocumentPathParams { vault_id }): Path<CreateDocumentPathParams>,
    Extension(user): Extension<User>,
    TypedHeader(device_id): TypedHeader<DeviceIdHeader>,
    State(state): State<AppState>,
    TypedMultipart(request): TypedMultipart<CreateDocumentVersion>,
) -> Result<Json<DocumentVersionWithoutContent>, SyncServerError> {
    let mut transaction = state
        .database
        .create_write_transaction(&vault_id)
        .await
        .map_err(server_error)?;

    let document_id = match request.document_id {
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

    let sanitized_relative_path = sanitize_path(&request.relative_path);

    let new_version = StoredDocumentVersion {
        vault_update_id: last_update_id + 1,
        document_id,
        relative_path: sanitized_relative_path,
        content: request.content.contents.to_vec(),
        updated_date: chrono::Utc::now(),
        is_deleted: false,
        user_id: user.name,
        device_id: device_id.0,
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

    Ok(Json(new_version.into()))
}
