use aide_axum_typed_multipart::FieldData;
use axum::body::Bytes;
use axum_typed_multipart::TryFromMultipart;
use schemars::JsonSchema;
use serde::{self, Deserialize};

use crate::app_state::database::models::{DeviceId, DocumentId, VaultUpdateId};

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateDocumentVersion {
    /// The client can decide the document id (if it wishes to) in order
    /// to help with syncing. If the client does not provide a document id,
    /// the server will generate one. If the client provides a document id
    /// it must not already exist in the database.
    pub document_id: Option<DocumentId>,
    pub relative_path: String,
    pub content_base64: String,
    pub device_id: Option<DeviceId>,
}

#[derive(Debug, TryFromMultipart, JsonSchema)]
pub struct CreateDocumentVersionMultipart {
    pub document_id: Option<DocumentId>,
    pub relative_path: String,
    #[form_data(limit = "unlimited")]
    pub content: FieldData<Bytes>,
    pub device_id: Option<DeviceId>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDocumentVersion {
    pub parent_version_id: VaultUpdateId,
    pub relative_path: String,
    pub content_base64: String,
    pub device_id: Option<DeviceId>,
}

#[derive(Debug, TryFromMultipart, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDocumentVersionMultipart {
    pub parent_version_id: VaultUpdateId,
    pub relative_path: String,
    #[form_data(limit = "unlimited")]
    pub content: FieldData<Bytes>,
    pub device_id: Option<DeviceId>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeleteDocumentVersion {
    pub relative_path: String,
    pub device_id: Option<DeviceId>,
}
