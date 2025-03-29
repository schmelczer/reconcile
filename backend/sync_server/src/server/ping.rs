use axum::{Json, extract::State};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};

use super::{auth::auth, responses::PingResponse};
use crate::{app_state::AppState, errors::SyncServerError};

#[axum::debug_handler]
pub async fn ping(
    maybe_auth_header: Option<TypedHeader<Authorization<Bearer>>>,
    State(state): State<AppState>,
) -> Result<Json<PingResponse>, SyncServerError> {
    let is_authenticated =
        maybe_auth_header.is_some_and(|auth_header| auth(&state, auth_header.token()).is_ok());

    Ok(Json(PingResponse {
        server_version: env!("CARGO_PKG_VERSION").to_owned(),
        is_authenticated,
    }))
}
