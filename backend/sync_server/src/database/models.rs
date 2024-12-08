use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::Serialize;
use sync_lib::bytes_to_base64;

pub type VaultId = String;
pub type DocumentId = uuid::Uuid;
pub type DocumentVersionId = i64;

#[derive(Debug, Clone)]
pub struct StoredDocumentVersion {
    pub vault_id: VaultId,
    pub document_id: DocumentId,
    pub version_id: DocumentVersionId,
    pub created_date: DateTime<Utc>,
    pub updated_date: DateTime<Utc>,
    pub relative_path: String,
    pub content: Vec<u8>,
    pub is_binary: bool,
    pub is_deleted: bool,
}

impl StoredDocumentVersion {
    pub fn content_as_string(&self) -> String { String::from_utf8_lossy(&self.content).to_string() }
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DocumentVersionWithoutContent {
    pub vault_id: VaultId,
    pub document_id: DocumentId,
    pub version_id: DocumentVersionId,
    pub created_date: DateTime<Utc>,
    pub updated_date: DateTime<Utc>,
    pub relative_path: String,
    pub is_binary: bool,
    pub is_deleted: bool,
}

impl From<StoredDocumentVersion> for DocumentVersionWithoutContent {
    fn from(value: StoredDocumentVersion) -> Self {
        Self {
            vault_id: value.vault_id,
            document_id: value.document_id,
            version_id: value.version_id,
            created_date: value.created_date,
            updated_date: value.updated_date,
            relative_path: value.relative_path,
            is_binary: value.is_binary,
            is_deleted: value.is_deleted,
        }
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DocumentVersion {
    pub vault_id: VaultId,
    pub document_id: DocumentId,
    pub version_id: DocumentVersionId,
    pub created_date: DateTime<Utc>,
    pub updated_date: DateTime<Utc>,
    pub relative_path: String,
    pub content_base64: String,
    pub is_binary: bool,
    pub is_deleted: bool,
}

impl From<StoredDocumentVersion> for DocumentVersion {
    fn from(value: StoredDocumentVersion) -> Self {
        Self {
            vault_id: value.vault_id,
            document_id: value.document_id,
            version_id: value.version_id,
            created_date: value.created_date,
            updated_date: value.updated_date,
            relative_path: value.relative_path,
            content_base64: bytes_to_base64(&value.content),
            is_binary: value.is_binary,
            is_deleted: value.is_deleted,
        }
    }
}
