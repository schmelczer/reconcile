use axum::extract::{Path, Query, State};
use axum_jsonschema::Json;
use schemars::JsonSchema;
use serde::Deserialize;

use super::responses::FetchLatestDocumentsResponse;
use crate::{
    app_state::{
        AppState,
        database::models::{VaultId, VaultUpdateId},
    },
    errors::{SyncServerError, server_error},
    utils::normalize::normalize,
};

// This is required for aide to infer the path parameter types and names
#[derive(Deserialize, JsonSchema)]
pub struct FetchLatestDocumentsPathParams {
    #[serde(deserialize_with = "normalize")]
    vault_id: VaultId,
}

// This is required for aide to infer the path parameter types and names
#[derive(Deserialize, JsonSchema)]
pub struct QueryParams {
    since_update_id: Option<VaultUpdateId>,
}

#[axum::debug_handler]
pub async fn fetch_latest_documents(
    Path(FetchLatestDocumentsPathParams { vault_id }): Path<FetchLatestDocumentsPathParams>,
    Query(QueryParams { since_update_id }): Query<QueryParams>,
    State(state): State<AppState>,
) -> Result<Json<FetchLatestDocumentsResponse>, SyncServerError> {
    let documents = if let Some(since_update_id) = since_update_id {
        state
            .database
            .get_latest_documents_since(&vault_id, since_update_id, None)
            .await
            .map_err(server_error)
    } else {
        state
            .database
            .get_latest_documents(&vault_id, None)
            .await
            .map_err(server_error)
    }?;

    Ok(Json(FetchLatestDocumentsResponse {
        last_update_id: documents
            .iter()
            .map(|doc| doc.vault_update_id)
            .max()
            .unwrap_or(since_update_id.unwrap_or(0)),
        latest_documents: documents,
    }))
}
