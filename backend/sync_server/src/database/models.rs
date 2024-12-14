use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::Serialize;
use sync_lib::bytes_to_base64;

pub type VaultId = String;
pub type DocumentVersionId = i64;

#[derive(Debug, Clone)]
pub struct StoredDocumentVersion {
    pub vault_id: VaultId,
    pub relative_path: String,
    pub version_id: DocumentVersionId,
    pub created_date: DateTime<Utc>,
    pub updated_date: DateTime<Utc>,
    pub content: Vec<u8>,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DocumentVersionWithoutContent {
    pub vault_id: VaultId,
    pub relative_path: String,
    pub version_id: DocumentVersionId,
    pub created_date: DateTime<Utc>,
    pub updated_date: DateTime<Utc>,
    pub is_deleted: bool,
}

impl From<StoredDocumentVersion> for DocumentVersionWithoutContent {
    fn from(value: StoredDocumentVersion) -> Self {
        Self {
            vault_id: value.vault_id,
            relative_path: value.relative_path,
            version_id: value.version_id,
            created_date: value.created_date,
            updated_date: value.updated_date,
            is_deleted: value.is_deleted,
        }
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PingResponse {
    pub server_version: String,
    pub is_authenticated: bool,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DocumentVersion {
    pub vault_id: VaultId,
    pub relative_path: String,
    pub version_id: DocumentVersionId,
    pub created_date: DateTime<Utc>,
    pub updated_date: DateTime<Utc>,
    pub content_base64: String,
    pub is_deleted: bool,
}

impl From<StoredDocumentVersion> for DocumentVersion {
    fn from(value: StoredDocumentVersion) -> Self {
        Self {
            vault_id: value.vault_id,
            relative_path: value.relative_path,
            version_id: value.version_id,
            created_date: value.created_date,
            updated_date: value.updated_date,
            content_base64: bytes_to_base64(&value.content),
            is_deleted: value.is_deleted,
        }
    }
}
