use axum::{
    extract::{Path, Query, State},
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use schemars::JsonSchema;
use serde::Deserialize;

use super::{app_state::AppState, auth::auth, responses::FetchLatestDocumentsResponse};
use crate::{
    database::models::{VaultId, VaultUpdateId},
    errors::{server_error, SyncServerError},
};

// This is required for aide to infer the path parameter types and names
#[derive(Deserialize, JsonSchema)]
pub struct PathParams {
    vault_id: VaultId,
}

// This is required for aide to infer the path parameter types and names
#[derive(Deserialize, JsonSchema)]
pub struct QueryParams {
    since_update_id: Option<VaultUpdateId>,
}

#[axum::debug_handler]
pub async fn fetch_latest_documents(
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    Path(PathParams { vault_id }): Path<PathParams>,
    Query(QueryParams { since_update_id }): Query<QueryParams>,
    State(state): State<AppState>,
) -> Result<Json<FetchLatestDocumentsResponse>, SyncServerError> {
    auth(&state, auth_header.token())?;

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
