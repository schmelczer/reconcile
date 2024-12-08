use crate::app_state::AppState;
use crate::database::models::DocumentId;
use crate::database::models::DocumentVersionWithoutContent;
use crate::database::models::StoredDocumentVersion;
use crate::database::models::VaultId;
use crate::errors::client_error;
use crate::errors::not_found_error;
use crate::errors::server_error;
use crate::errors::SyncServerError;
use anyhow::anyhow;
use anyhow::Context;
use axum::extract::Path;
use axum::extract::State;
use axum::Json;
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use sync_lib::base64_to_bytes;
use sync_lib::base64_to_string;

use super::auth::auth;
use super::requests::UpdateDocumentVersion;

#[axum::debug_handler]
pub async fn update_document(
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    Path((vault_id, document_id)): Path<(VaultId, DocumentId)>,
    State(state): State<AppState>,
    Json(request): Json<UpdateDocumentVersion>,
) -> Result<Json<DocumentVersionWithoutContent>, SyncServerError> {
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

    let merged_content = if request.is_binary {
        base64_to_bytes(&request.content_base64)
            .context("Failed to decode base64 content in request")
            .map_err(client_error)?
    } else {
        reconcile::reconcile(
            &parent.content_as_string(),
            &latest_version.content_as_string(),
            &base64_to_string(&request.content_base64)
                .context("Failed to decode base64 content in request")
                .map_err(client_error)?,
        )
        .into_bytes()
    };

    let new_version = StoredDocumentVersion {
        vault_id,
        document_id,
        version_id: latest_version.version_id + 1,
        content: merged_content,
        created_date: request.created_date,
        relative_path: request.relative_path,
        updated_date: chrono::Utc::now(),
        is_binary: request.is_binary,
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
