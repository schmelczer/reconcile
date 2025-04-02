use log::info;

use crate::{
    app_state::{AppState, database::models::VaultId},
    config::user_config::{AllowListedVaults, User, VaultAccess},
    errors::{SyncServerError, permission_denied_error, unauthenticated_error},
};

// TODO: turn this into a middleware
pub fn auth(app_state: &AppState, token: &str, vault: &VaultId) -> Result<User, SyncServerError> {
    let user = app_state
        .config
        .users
        .get_user(token)
        .cloned()
        .ok_or_else(|| unauthenticated_error(anyhow::anyhow!("Invalid token")))?;

    info!("User `{}` authenticated", user.name);

    if match user.vault_access {
        VaultAccess::AllowAccessToAll => true,
        VaultAccess::AllowList(AllowListedVaults { ref allowed }) => allowed.contains(vault),
    } {
        info!(
            "User `{}` is authorised to access to vault `{}`",
            user.name, vault
        );
        Ok(user)
    } else {
        Err(permission_denied_error(anyhow::anyhow!(
            "Permission denied for vault `{vault}`"
        )))
    }
}
