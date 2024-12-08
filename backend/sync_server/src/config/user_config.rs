use std::path::Path;

use anyhow::{Context, Result};
use log::debug;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{
    consts::{DEFAULT_HOST, DEFAULT_MAX_CONNECTIONS, DEFAULT_PORT, DEFAULT_SQLITE_URL},
    errors::SyncServerError,
};
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserConfig {
    #[serde(default = "Vec::new")]
    pub users: Vec<User>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub name: String,
    pub token: String,
}
