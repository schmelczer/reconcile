use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use tokio::sync::{Mutex, broadcast};

use super::database::models::{DeviceId, DocumentVersionWithoutContent, VaultId};
use crate::{config::server_config::ServerConfig, errors::server_error};

#[derive(Debug, Clone)]
pub struct Broadcasts {
    max_clients_per_vault: usize,
    tx: Arc<Mutex<HashMap<VaultId, broadcast::Sender<VaultUpdate>>>>,
}

#[derive(Debug, Clone)]
pub struct VaultUpdate {
    pub origin_device_id: Option<DeviceId>,
    pub document: DocumentVersionWithoutContent,
}

impl Broadcasts {
    pub fn new(server_config: &ServerConfig) -> Self {
        Self {
            max_clients_per_vault: server_config.max_clients_per_vault,
            tx: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_receiver(&self, vault: VaultId) -> broadcast::Receiver<VaultUpdate> {
        let tx = self.get_or_create(vault).await;

        tx.subscribe()
    }

    /// Sent a document update to all clients subscribed to the vault.
    /// We ignore & log failures.
    pub async fn send(&self, vault: VaultId, document: VaultUpdate) {
        let tx = self.get_or_create(vault).await;

        let result = tx
            .send(document)
            .context("Cannot broadcast update message to websocket listeners")
            .map_err(server_error);

        if result.is_err() {
            log::debug!("Failed to send message: {result:?}");
        }
    }

    async fn get_or_create(&self, vault: VaultId) -> broadcast::Sender<VaultUpdate> {
        let mut tx = self.tx.lock().await;

        tx.entry(vault)
            .or_insert_with(|| broadcast::channel(self.max_clients_per_vault).0.clone())
            .clone()
    }
}
