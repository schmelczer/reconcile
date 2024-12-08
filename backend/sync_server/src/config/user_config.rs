use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserConfig {
    #[serde(default = "default_users")]
    pub user_tokens: Vec<User>,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub name: String,
    pub token: String,
}

impl Default for UserConfig {
    fn default() -> Self {
        UserConfig {
            user_tokens: default_users(),
        }
    }
}

fn default_users() -> Vec<User> {
    vec![User {
        name: "admin".to_string(),
        token: get_random_token(),
    }]
}

pub fn get_random_token() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}
