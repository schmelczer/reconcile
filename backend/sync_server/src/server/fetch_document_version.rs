use anyhow::anyhow;
use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;

use crate::{
    app_state::{
        AppState,
        database::models::{DocumentId, DocumentVersion, VaultId, VaultUpdateId},
    },
    errors::{SyncServerError, not_found_error, server_error},
    utils::normalize::normalize,
};

#[derive(Deserialize)]
pub struct FetchDocumentVersionPathParams {
    #[serde(deserialize_with = "normalize")]
    vault_id: VaultId,

    document_id: DocumentId,
    vault_update_id: VaultUpdateId,
}

#[axum::debug_handler]
pub async fn fetch_document_version(
    Path(FetchDocumentVersionPathParams {
        vault_id,
        document_id,
        vault_update_id,
    }): Path<FetchDocumentVersionPathParams>,
    State(state): State<AppState>,
) -> Result<Json<DocumentVersion>, SyncServerError> {
    let result = state
        .database
        .get_document_version(&vault_id, vault_update_id, None)
        .await
        .map_err(server_error)?
        .map_or_else(
            || {
                Err(not_found_error(anyhow!(
                    "Document with vault update id `{vault_update_id}` not found",
                )))
            },
            Ok,
        )?;

    if result.document_id != document_id {
        return Err(not_found_error(anyhow!(
            "Document with document id `{document_id}` does not have a version with id \
             `{vault_update_id}`",
        )));
    }

    Ok(Json(result.into()))
}
