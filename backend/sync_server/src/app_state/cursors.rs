use core::time::Duration;
use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use super::{
    database::models::{DeviceId, VaultId},
    websocket::{
        broadcasts::Broadcasts,
        models::{
            ClientCursors, CursorPositionFromServer, CursorSpan, WebSocketServerMessage,
            WebSocketServerMessageWithOrigin,
        },
    },
};
use crate::config::database_config::DatabaseConfig;

#[derive(Clone, Debug)]
pub struct Cursors {
    config: DatabaseConfig,
    broadcasts: Broadcasts,
    vault_to_cursors: Arc<Mutex<HashMap<VaultId, Vec<ClientCursorsWithTimeToLive>>>>,
}

impl Cursors {
    pub fn new(config: &DatabaseConfig, broadcasts: &Broadcasts) -> Self {
        Self {
            config: config.clone(),
            broadcasts: broadcasts.clone(),
            vault_to_cursors: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn update_cursors(
        &self,
        vault_id: VaultId,
        user_name: String,
        device_id: &DeviceId,
        document_to_cursors: HashMap<String, Vec<CursorSpan>>,
    ) {
        let mut vault_to_cursors = self.vault_to_cursors.lock().await;

        let all_device_cursors = vault_to_cursors.entry(vault_id).or_insert_with(Vec::new);

        all_device_cursors.retain(|c| &c.client_cursors.device_id != device_id);
        all_device_cursors.push(ClientCursorsWithTimeToLive::new(ClientCursors {
            user_name,
            device_id: device_id.to_string(),
            cursors: document_to_cursors,
        }));

        drop(vault_to_cursors); // Explicitly drop the lock before broadcasting to avoid deadlock
        self.broadcast_cursors().await;
    }

    pub async fn get_cursors(&self, vault_id: &VaultId) -> Vec<ClientCursors> {
        let vault_to_cursors = self.vault_to_cursors.lock().await;
        vault_to_cursors
            .get(vault_id)
            .map(|cursors| {
                cursors
                    .iter()
                    .cloned()
                    .map(|with_ttl| with_ttl.client_cursors)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }

    pub fn start_background_task(self) {
        tokio::spawn(async move {
            loop {
                self.remove_expired_cursors().await;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
    }

    async fn remove_expired_cursors(&self) {
        let mut vault_to_cursors = self.vault_to_cursors.lock().await;

        for (_vault_id, cursors) in vault_to_cursors.iter_mut() {
            cursors.retain(|cursor| !cursor.is_expired(self.config.cursor_timeout));
        }
    }

    async fn broadcast_cursors(&self) {
        let vault_to_cursors = self.vault_to_cursors.lock().await;

        for (vault_id, cursors) in vault_to_cursors.iter() {
            self.broadcasts
                .send_document_update(
                    vault_id.clone(),
                    WebSocketServerMessageWithOrigin::new(WebSocketServerMessage::CursorPositions(
                        CursorPositionFromServer {
                            clients: cursors.iter().map(|c| c.client_cursors.clone()).collect(),
                        },
                    )),
                )
                .await;
        }
    }

    pub async fn remove_cursors_of_device(&self, vault_id: &str, device_id: &str) {
        let mut vault_to_cursors = self.vault_to_cursors.lock().await;

        if let Some(cursors) = vault_to_cursors.get_mut(vault_id) {
            cursors.retain(|c| c.client_cursors.device_id != device_id);
        }
    }
}

#[derive(Clone, Debug)]
struct ClientCursorsWithTimeToLive {
    client_cursors: ClientCursors,
    last_updated: std::time::Instant,
}

impl ClientCursorsWithTimeToLive {
    fn new(client_cursors: ClientCursors) -> Self {
        Self {
            client_cursors,
            last_updated: std::time::Instant::now(),
        }
    }

    pub fn is_expired(&self, ttl: Duration) -> bool { self.last_updated.elapsed() > ttl }
}
