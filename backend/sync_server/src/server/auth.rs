use std::collections::HashMap;

use axum::{
    extract::{Path, Request, State},
    middleware::Next,
    response::Response,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use log::info;

use crate::{
    app_state::{AppState, database::models::VaultId},
    config::user_config::{AllowListedVaults, User, VaultAccess},
    errors::{SyncServerError, permission_denied_error, unauthenticated_error},
    utils::normalize::normalize_string,
};

pub async fn auth_middleware(
    State(state): State<AppState>,
    Path(path_params): Path<HashMap<String, String>>,
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    mut req: Request,
    next: Next,
) -> Result<Response, SyncServerError> {
    let token = auth_header.token().trim();
    let vault_id = normalize_string(
        path_params
            .get("vault_id")
            .ok_or_else(|| unauthenticated_error(anyhow::anyhow!("Missing vault_id")))?,
    );

    let user = auth(&state, token, &vault_id)?;

    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}

pub fn auth(state: &AppState, token: &str, vault_id: &VaultId) -> Result<User, SyncServerError> {
    let user = state
        .config
        .users
        .get_user(token)
        .cloned()
        .ok_or_else(|| unauthenticated_error(anyhow::anyhow!("Invalid token")))?;

    if match user.vault_access {
        VaultAccess::AllowAccessToAll => true,
        VaultAccess::AllowList(AllowListedVaults { ref allowed }) => allowed.contains(vault_id),
    } {
        info!(
            "User '{}' is authenticated and is authorised to access to vault '{vault_id}'",
            user.name
        );

        Ok(user)
    } else {
        info!(
            "User '{}' is authenticated but is not authorised to access vault '{vault_id}'",
            user.name
        );

        Err(permission_denied_error(anyhow::anyhow!(
            "Permission denied for vault `{vault_id}`"
        )))
    }
}
