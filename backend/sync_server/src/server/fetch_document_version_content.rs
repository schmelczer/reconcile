use anyhow::anyhow;
use axum::{
    body::Bytes,
    extract::{Path, State},
};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    app_state::{
        AppState,
        database::models::{DocumentId, VaultId, VaultUpdateId},
    },
    errors::{SyncServerError, not_found_error, server_error},
    utils::normalize::normalize,
};

// This is required for aide to infer the path parameter types and names
#[derive(Deserialize, JsonSchema)]
pub struct FetchDocumentVersionContentPathParams {
    #[serde(deserialize_with = "normalize")]
    vault_id: VaultId,

    document_id: DocumentId,
    vault_update_id: VaultUpdateId,
}

#[axum::debug_handler]
pub async fn fetch_document_version_content(
    Path(FetchDocumentVersionContentPathParams {
        vault_id,
        document_id,
        vault_update_id,
    }): Path<FetchDocumentVersionContentPathParams>,
    State(state): State<AppState>,
) -> Result<Bytes, SyncServerError> {
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

    Ok(result.content.into())
}
