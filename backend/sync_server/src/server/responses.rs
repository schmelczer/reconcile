use schemars::JsonSchema;
use serde::{self, Serialize};

use crate::database::models::{DocumentVersion, DocumentVersionWithoutContent, VaultUpdateId};

/// Response to a ping request.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PingResponse {
    /// Semantic version of the server.
    pub server_version: String,

    /// Whether the client is authenticated based on the sent Authorization
    /// header.
    pub is_authenticated: bool,
}

/// Response to a fetch latest documents request.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FetchLatestDocumentsResponse {
    pub latest_documents: Vec<DocumentVersionWithoutContent>,

    /// The update ID of the latest document in the response.
    pub last_update_id: VaultUpdateId,
}

/// Response to an update document request.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(tag = "type")]
pub enum DocumentUpdateResponse {
    /// Returned when the created/updated document's content is the same as was
    /// sent in the create/update request and thus the response doesn't contain
    /// the content because the client must already have it.
    FastForwardUpdate(DocumentVersionWithoutContent),

    /// Returned when the created/updated document's content is different from
    /// what was sent in the create/update request.
    MergingUpdate(DocumentVersion),
}
