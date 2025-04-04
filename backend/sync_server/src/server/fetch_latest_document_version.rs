use anyhow::anyhow;
use axum::extract::{Path, State};
use axum_jsonschema::Json;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    app_state::{
        AppState,
        database::models::{DocumentId, DocumentVersion, VaultId},
    },
    errors::{SyncServerError, not_found_error, server_error},
};

// This is required for aide to infer the path parameter types and names
#[derive(Deserialize, JsonSchema)]
pub struct FetchLatestDocumentVersionPathParams {
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
