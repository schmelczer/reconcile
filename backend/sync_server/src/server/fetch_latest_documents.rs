use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use super::auth::auth;
use crate::{
    app_state::AppState,
    database::models::{DocumentVersionWithoutContent, VaultId},
    errors::{server_error, SyncServerError},
};

#[axum::debug_handler]
pub async fn fetch_latest_documents(
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    Path(vault_id): Path<VaultId>,
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
