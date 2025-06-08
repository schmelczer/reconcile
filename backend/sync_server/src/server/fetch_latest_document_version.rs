use anyhow::anyhow;
use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;

use crate::{
    app_state::{
        AppState,
        database::models::{DocumentId, DocumentVersion, VaultId},
    },
    errors::{SyncServerError, not_found_error, server_error},
    utils::normalize::normalize,
};

#[derive(Deserialize)]
pub struct FetchLatestDocumentVersionPathParams {
    #[serde(deserialize_with = "normalize")]
    vault_id: VaultId,

    document_id: DocumentId,
}

#[axum::debug_handler]
pub async fn fetch_latest_document_version(
    Path(FetchLatestDocumentVersionPathParams {
        vault_id,
        document_id,
    }): Path<FetchLatestDocumentVersionPathParams>,
    State(state): State<AppState>,
) -> Result<Json<DocumentVersion>, SyncServerError> {
    let latest_version = state
        .database
        .get_latest_document(&vault_id, &document_id, None)
        .await
        .map_err(server_error)?
        .map_or_else(
            || {
                Err(not_found_error(anyhow!(
                    "Document with id `{document_id}` not found",
                )))
            },
            Ok,
        )?;

    Ok(Json(latest_version.into()))
}
