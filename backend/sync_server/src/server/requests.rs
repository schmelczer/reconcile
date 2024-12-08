use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateDocumentVersion {
    pub created_date: DateTime<Utc>,
    pub relative_path: String,
    pub content_base64: String,
    pub is_binary: bool,
}

#[derive(Debug, Deserialize)]
pub struct DeleteDocumentVersion {
    pub created_date: DateTime<Utc>,
}
