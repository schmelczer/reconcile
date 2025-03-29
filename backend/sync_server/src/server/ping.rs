use axum::{
    Json,
    extract::{Path, State},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use schemars::JsonSchema;
use serde::Deserialize;

use super::{auth::auth, responses::PingResponse};
use crate::{
    app_state::{AppState, database::models::VaultId},
    errors::SyncServerError,
};

// This is required for aide to infer the path parameter types and names
#[derive(Deserialize, JsonSchema)]
pub struct PingPathParams {
    vault_id: VaultId,
}

#[axum::debug_handler]
pub async fn ping(
    maybe_auth_header: Option<TypedHeader<Authorization<Bearer>>>,
    Path(PingPathParams { vault_id }): Path<PingPathParams>,
    State(state): State<AppState>,
) -> Result<Json<PingResponse>, SyncServerError> {
    let is_authenticated = maybe_auth_header
        .is_some_and(|auth_header| auth(&state, auth_header.token(), &vault_id).is_ok());

    Ok(Json(PingResponse {
        server_version: env!("CARGO_PKG_VERSION").to_owned(),
        is_authenticated,
    }))
}
