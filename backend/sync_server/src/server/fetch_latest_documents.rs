use crate::app_state::AppState;
use crate::database::models::DocumentVersionWithoutContent;
use crate::database::models::VaultId;
use crate::errors::server_error;
use crate::errors::SyncServerError;
use axum::extract::Path;
use axum::extract::State;
use axum::Json;

#[axum::debug_handler]
pub async fn fetch_latest_documents(
    Path(vault_id): Path<VaultId>,
    State(state): State<AppState>,
) -> Result<Json<Vec<DocumentVersionWithoutContent>>, SyncServerError> {
    let latest_version = state
        .database
        .get_latest_documents(&vault_id, None)
        .await
        .map_err(server_error)?;

    Ok(Json(latest_version))
}
