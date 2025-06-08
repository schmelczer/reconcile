use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::app_state::database::models::{DeviceId, DocumentVersionWithoutContent, VaultUpdateId};

#[derive(TS, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WebSocketHandshake {
    pub token: String,
    pub device_id: DeviceId,

    #[ts(as = "Option<i32>")]
    pub last_seen_vault_update_id: Option<VaultUpdateId>,
}

#[derive(TS, Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CursorSpan {
    pub start: usize,
    pub end: usize,
}

#[derive(TS, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CursorPositionFromClient {
    pub document_to_cursors: HashMap<String, Vec<CursorSpan>>,
}

#[derive(TS, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClientCursors {
    pub user_name: String,
    pub device_id: DeviceId,
    pub cursors: HashMap<String, Vec<CursorSpan>>,
}

#[derive(TS, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CursorPositionFromServer {
    pub clients: Vec<ClientCursors>,
}

#[derive(TS, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WebSocketVaultUpdate {
    pub documents: Vec<DocumentVersionWithoutContent>,
    pub is_initial_sync: bool,
}

#[derive(TS, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", tag = "type")]
#[ts(export)]
pub enum WebSocketClientMessage {
    Handshake(WebSocketHandshake),
    CursorPositions(CursorPositionFromClient),
}

#[derive(TS, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase", tag = "type")]
#[ts(export)]
pub enum WebSocketServerMessage {
    VaultUpdate(WebSocketVaultUpdate),
    CursorPositions(CursorPositionFromServer),
}

#[derive(Clone, Debug)]
pub struct WebSocketServerMessageWithOrigin {
    pub origin_device_id: Option<DeviceId>,
    pub message: WebSocketServerMessage,
}

impl WebSocketServerMessageWithOrigin {
    pub fn new(message: WebSocketServerMessage) -> Self {
        Self {
            origin_device_id: None,
            message,
        }
    }

    pub fn with_origin(origin_device_id: DeviceId, message: WebSocketServerMessage) -> Self {
        Self {
            origin_device_id: Some(origin_device_id),
            message,
        }
    }
}
