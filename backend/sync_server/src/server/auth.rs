use crate::{
    app_state::AppState,
    config::user_config::User,
    errors::{SyncServerError, unauthorized_error},
};

// TODO: turn this into a middleware
pub fn auth(app_state: &AppState, token: &str) -> Result<User, SyncServerError> {
    app_state
        .config
        .users
        .get_user(token)
        .cloned()
        .ok_or_else(|| unauthorized_error(anyhow::anyhow!("Invalid token")))
}
