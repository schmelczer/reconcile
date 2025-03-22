use anyhow::anyhow;
use axum::{
    body::Bytes,
    extract::{Path, State},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use schemars::JsonSchema;
use serde::Deserialize;

use super::{app_state::AppState, auth::auth};
use crate::{
    database::models::{DocumentId, VaultId, VaultUpdateId},
    errors::{SyncServerError, not_found_error, server_error},
};

// This is required for aide to infer the path parameter types and names
#[derive(Deserialize, JsonSchema)]
pub struct FetchDocumentVersionContentPathParams {
    vault_id: VaultId,
    document_id: DocumentId,
    vault_update_id: VaultUpdateId,
}

#[axum::debug_handler]
pub async fn fetch_document_version_content(
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    Path(FetchDocumentVersionContentPathParams {
        vault_id,
        document_id,
        vault_update_id,
    }): Path<FetchDocumentVersionContentPathParams>,
    State(mut state): State<AppState>,
) -> Result<Bytes, SyncServerError> {
    auth(&state, auth_header.token())?;

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
