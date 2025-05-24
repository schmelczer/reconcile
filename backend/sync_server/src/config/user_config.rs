use bimap::BiHashMap;
use rand::{Rng, distr::Alphanumeric, rng};
use serde::{Deserialize, Deserializer, Serialize, de::Error};

use crate::app_state::database::models::VaultId;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserConfig {
    #[serde(default = "default_users", deserialize_with = "validate_users")]
    pub user_configs: Vec<User>,
}

fn validate_users<'de, D>(deserializer: D) -> Result<Vec<User>, D::Error>
where
    D: Deserializer<'de>,
{
    let users = Vec::<User>::deserialize(deserializer)?;

    let mut user_token_map = BiHashMap::new();
    for user in &users {
        if let Some(existing_name) = user_token_map.get_by_right(&user.token) {
            return Err(D::Error::custom(format!(
                "Duplicate user token found: '{}' for users '{}' and '{}'. User tokens must be \
                 unique.",
                user.token, existing_name, user.name
            )));
        }

        if user_token_map.contains_left(&user.name) {
            return Err(D::Error::custom(format!(
                "Duplicate user name found: '{}'. User names must be unique.",
                user.name
            )));
        }

        user_token_map.insert(user.name.clone(), user.token.clone());
    }

    Ok(users)
}

impl UserConfig {
    pub fn get_user(&self, token: &str) -> Option<&User> {
        self.user_configs.iter().find(|u| u.token == token)
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
            user_configs: default_users(),
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
    rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}
#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_validate_users_unique_names_and_tokens() {
        let config_json = json!({
            "user_configs": [
                {
                    "name": "alice",
                    "token": "token1",
                    "vault_access": { "type": "allow_access_to_all" }
                },
                {
                    "name": "bob",
                    "token": "token2",
                    "vault_access": { "type": "allow_access_to_all" }
                }
            ]
        });

        let config: Result<UserConfig, _> = serde_json::from_value(config_json);
        assert!(config.is_ok());
    }

    #[test]
    fn test_validate_users_duplicate_names() {
        let config_json = json!({
            "user_configs": [
                {
                    "name": "alice",
                    "token": "token1",
                    "vault_access": { "type": "allow_access_to_all" }
                },
                {
                    "name": "alice",
                    "token": "token2",
                    "vault_access": { "type": "allow_access_to_all" }
                }
            ]
        });

        let config: Result<UserConfig, _> = serde_json::from_value(config_json);
        assert!(config.is_err());
        let err = config.unwrap_err().to_string();
        assert!(err.contains("Duplicate user name found"));
    }

    #[test]
    fn test_validate_users_duplicate_tokens() {
        let config_json = json!({
            "user_configs": [
                {
                    "name": "alice",
                    "token": "token1",
                    "vault_access": { "type": "allow_access_to_all" }
                },
                {
                    "name": "bob",
                    "token": "token1",
                    "vault_access": { "type": "allow_access_to_all" }
                }
            ]
        });

        let config: Result<UserConfig, _> = serde_json::from_value(config_json);
        assert!(config.is_err());
        let err = config.unwrap_err().to_string();
        assert!(err.contains("Duplicate user token found"));
    }
}
