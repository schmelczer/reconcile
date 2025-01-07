use aide_axum_typed_multipart::FieldData;
use axum::body::Bytes;
use axum_typed_multipart::TryFromMultipart;
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

#[derive(Debug, TryFromMultipart, JsonSchema)]
pub struct CreateDocumentVersionMultipart {
    pub relative_path: String,
    pub created_date: DateTime<Utc>,
    #[form_data(limit = "unlimited")]
    pub content: FieldData<Bytes>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDocumentVersion {
    pub parent_version_id: VaultUpdateId,
    pub relative_path: String,
    pub created_date: DateTime<Utc>,
    pub content_base64: String,
}

#[derive(Debug, TryFromMultipart, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDocumentVersionMultipart {
    pub parent_version_id: VaultUpdateId,
    pub relative_path: String,
    pub created_date: DateTime<Utc>,
    #[form_data(limit = "unlimited")]
    pub content: FieldData<Bytes>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeleteDocumentVersion {
    pub relative_path: String,
    pub created_date: DateTime<Utc>,
}
