use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use tokio::sync::{Mutex, broadcast};

use super::models::WebSocketServerMessageWithOrigin;
use crate::{
    app_state::database::models::VaultId, config::server_config::ServerConfig, errors::server_error,
};

#[derive(Debug, Clone)]
pub struct Broadcasts {
    max_clients_per_vault: usize,
    tx: Arc<Mutex<HashMap<VaultId, broadcast::Sender<WebSocketServerMessageWithOrigin>>>>,
}

impl Broadcasts {
    pub fn new(server_config: &ServerConfig) -> Self {
        Self {
            max_clients_per_vault: server_config.max_clients_per_vault,
            tx: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_receiver(
        &self,
        vault: VaultId,
    ) -> broadcast::Receiver<WebSocketServerMessageWithOrigin> {
        let tx = self.get_or_create(vault).await;

        tx.subscribe()
    }

    /// Notify all clients (who are subscribed to the vault) about an update.
    /// We only log failures.
    pub async fn send_document_update(
        &self,
        vault: VaultId,
        document: WebSocketServerMessageWithOrigin,
    ) {
        let tx = self.get_or_create(vault).await;

        let result = tx
            .send(document)
            .context("Cannot broadcast server message to websocket listeners")
            .map_err(server_error);

        if result.is_err() {
            log::debug!("Failed to send message: {result:?}");
        }
    }

    async fn get_or_create(
        &self,
        vault: VaultId,
    ) -> broadcast::Sender<WebSocketServerMessageWithOrigin> {
        let mut tx = self.tx.lock().await;

        tx.entry(vault)
            .or_insert_with(|| broadcast::channel(self.max_clients_per_vault).0.clone())
            .clone()
    }
}
