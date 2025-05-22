use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::Serialize;
use sync_lib::bytes_to_base64;

pub type VaultId = String;
pub type VaultUpdateId = i64;
pub type DocumentId = uuid::Uuid;
pub type UserId = String;
pub type DeviceId = String;

#[derive(Debug, Clone)]
pub struct StoredDocumentVersion {
    pub vault_update_id: VaultUpdateId,
    pub document_id: DocumentId,
    pub relative_path: String,
    pub updated_date: DateTime<Utc>,
    pub content: Vec<u8>,
    pub is_deleted: bool,
    pub user_id: UserId,
    pub device_id: DeviceId,
}

impl PartialEq<Self> for StoredDocumentVersion {
    fn eq(&self, other: &Self) -> bool { self.vault_update_id == other.vault_update_id }
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DocumentVersionWithoutContent {
    pub vault_update_id: VaultUpdateId,
    pub document_id: DocumentId,
    pub relative_path: String,
    pub updated_date: DateTime<Utc>,
    pub is_deleted: bool,
    pub user_id: UserId,
    pub device_id: DeviceId,
}

impl From<StoredDocumentVersion> for DocumentVersionWithoutContent {
    fn from(value: StoredDocumentVersion) -> Self {
        Self {
            vault_update_id: value.vault_update_id,
            document_id: value.document_id,
            relative_path: value.relative_path,
            updated_date: value.updated_date,
            is_deleted: value.is_deleted,
            user_id: value.user_id,
            device_id: value.device_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DocumentVersion {
    pub vault_update_id: VaultUpdateId,
    pub document_id: DocumentId,
    pub relative_path: String,
    pub updated_date: DateTime<Utc>,
    pub content_base64: String,
    pub is_deleted: bool,
    pub user_id: UserId,
    pub device_id: DeviceId,
}

impl From<StoredDocumentVersion> for DocumentVersion {
    fn from(value: StoredDocumentVersion) -> Self {
        Self {
            vault_update_id: value.vault_update_id,
            document_id: value.document_id,
            relative_path: value.relative_path,
            updated_date: value.updated_date,
            content_base64: bytes_to_base64(&value.content),
            is_deleted: value.is_deleted,
            user_id: value.user_id,
            device_id: value.device_id,
        }
    }
}
