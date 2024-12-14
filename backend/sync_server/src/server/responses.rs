use schemars::JsonSchema;
use serde::{self, Serialize};

use crate::database::models::{DocumentVersionWithoutContent, VaultUpdateId};

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PingResponse {
    pub server_version: String,
    pub is_authenticated: bool,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FetchLatestDocumentsResponse {
    pub latest_documents: Vec<DocumentVersionWithoutContent>,
    pub last_update_id: VaultUpdateId,
}
