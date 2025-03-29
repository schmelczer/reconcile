use std::default;

use rand::{Rng as _, distributions::Alphanumeric, thread_rng};
use serde::{Deserialize, Serialize};

use crate::app_state::database::models::VaultId;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserConfig {
    #[serde(default = "default_users")]
    pub user_tokens: Vec<User>,
}

impl UserConfig {
    pub fn get_user(&self, token: &str) -> Option<&User> {
        self.user_tokens.iter().find(|u| u.token == token)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub name: String,
    pub token: String,
    pub vault_access: VaultAccess,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            user_tokens: default_users(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum VaultAccess {
    #[default]
    AllowAccessToAll,

    AllowList(AllowListedVaults),
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct AllowListedVaults {
    pub allowed: Vec<VaultId>,
}

fn default_users() -> Vec<User> {
    vec![User {
        name: "admin".to_owned(),
        token: get_random_token(),
        vault_access: VaultAccess::default(),
    }]
}

pub fn get_random_token() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}
