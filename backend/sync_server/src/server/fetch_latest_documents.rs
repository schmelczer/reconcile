use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use schemars::JsonSchema;
use serde::Deserialize;

use super::auth::auth;
use crate::{
    app_state::AppState,
    database::models::{DocumentVersionWithoutContent, VaultId},
    errors::{server_error, SyncServerError},
};

// This is required for aide to infer the path parameter types and names
#[derive(Deserialize, JsonSchema)]
pub struct PathParams {
    vault_id: VaultId,
}

#[axum::debug_handler]
pub async fn fetch_latest_documents(
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    Path(PathParams { vault_id }): Path<PathParams>,
    State(state): State<AppState>,
) -> Result<Json<Vec<DocumentVersionWithoutContent>>, SyncServerError> {
    auth(&state, auth_header.token())?;

    let latest_version = state
        .database
        .get_latest_documents(&vault_id, None)
        .await
        .map_err(server_error)?;

    Ok(Json(latest_version))
}
