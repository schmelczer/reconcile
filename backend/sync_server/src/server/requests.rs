use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{self, Deserialize};

use crate::database::models::VaultUpdateId;

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateDocumentVersion {
    pub relative_path: String,
    pub created_date: DateTime<Utc>,
    pub content_base64: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDocumentVersion {
    pub parent_version_id: VaultUpdateId,
    pub relative_path: String,
    pub created_date: DateTime<Utc>,
    pub content_base64: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeleteDocumentVersion {
    pub relative_path: String,
    pub created_date: DateTime<Utc>,
}
