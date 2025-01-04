use super::app_state::AppState;
use crate::{
    config::user_config::User,
    errors::{unauthorized_error, SyncServerError},
};

pub fn auth(app_state: &AppState, token: &str) -> Result<User, SyncServerError> {
    app_state
        .config
        .users
        .get_user(token)
        .cloned()
        .ok_or_else(|| unauthorized_error(anyhow::anyhow!("Invalid token")))
}
