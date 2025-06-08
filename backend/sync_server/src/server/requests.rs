use axum::body::Bytes;
use axum_typed_multipart::{FieldData, TryFromMultipart};
use serde::{self, Deserialize};
use ts_rs::TS;

use crate::app_state::database::models::{DocumentId, VaultUpdateId};

#[derive(TS, Debug, TryFromMultipart)]
#[ts(export)]
pub struct CreateDocumentVersion {
    /// The client can decide the document id (if it wishes to) in order
    /// to help with syncing. If the client does not provide a document id,
    /// the server will generate one. If the client provides a document id
    /// it must not already exist in the database.
    pub document_id: Option<DocumentId>,
    pub relative_path: String,

    #[ts(as = "Vec<u8>")]
    #[form_data(limit = "unlimited")]
    pub content: FieldData<Bytes>,
}

#[derive(TS, Debug, TryFromMultipart)]
#[ts(export)]
pub struct UpdateDocumentVersion {
    pub parent_version_id: VaultUpdateId,
    pub relative_path: String,

    #[ts(as = "Vec<u8>")]
    #[form_data(limit = "unlimited")]
    pub content: FieldData<Bytes>,
}

#[derive(TS, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DeleteDocumentVersion {
    pub relative_path: String,
}
