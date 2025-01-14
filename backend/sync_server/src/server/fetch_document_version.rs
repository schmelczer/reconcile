use anyhow::anyhow;
use axum::extract::{Path, State};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use axum_jsonschema::Json;
use schemars::JsonSchema;
use serde::Deserialize;

use super::{app_state::AppState, auth::auth};
use crate::{
    database::models::{DocumentId, DocumentVersion, VaultId, VaultUpdateId},
    errors::{SyncServerError, not_found_error, server_error},
};

// This is required for aide to infer the path parameter types and names
#[derive(Deserialize, JsonSchema)]
pub struct PathParams {
    vault_id: VaultId,
    document_id: DocumentId,
    vault_update_id: VaultUpdateId,
}

#[axum::debug_handler]
pub async fn fetch_document_version(
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    Path(PathParams {
        vault_id,
        document_id,
        vault_update_id,
    }): Path<PathParams>,
    State(state): State<AppState>,
) -> Result<Json<DocumentVersion>, SyncServerError> {
    auth(&state, auth_header.token())?;

    let result = state
        .database
        .get_document_version(&vault_id, vault_update_id, None)
        .await
        .map_err(server_error)?
        .map(Ok)
        .unwrap_or_else(|| {
            Err(not_found_error(anyhow!(
                "Document with vault update id `{vault_update_id}` not found",
            )))
        })?;

    if result.document_id != document_id {
        return Err(not_found_error(anyhow!(
            "Document with document id `{document_id}` does not have a version with id \
             `{vault_update_id}`",
        )));
    }

    Ok(Json(result.into()))
}
