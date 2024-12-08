use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateDocumentVersion {
    pub created_date: DateTime<Utc>,
    pub relative_path: String,
    pub content_base64: String,
    pub is_binary: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteDocumentVersion {
    pub created_date: DateTime<Utc>,
}
