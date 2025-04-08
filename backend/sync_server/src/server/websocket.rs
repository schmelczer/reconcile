use anyhow::Context;
use axum::{
    extract::{
        Path, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use futures::{
    sink::SinkExt,
    stream::{SplitSink, StreamExt},
};
use log::{error, info, warn};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::auth::auth;
use crate::{
    app_state::{
        AppState,
        database::models::{DeviceId, DocumentVersionWithoutContent, VaultId, VaultUpdateId},
    },
    errors::{SyncServerError, server_error, unauthenticated_error},
    utils::normalize::normalize,
};

// This is required for aide to infer the path parameter types and names
#[derive(Deserialize, JsonSchema)]
pub struct WebsocketPathParams {
    #[serde(deserialize_with = "normalize")]
    vault_id: VaultId,
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path(WebsocketPathParams { vault_id }): Path<WebsocketPathParams>,
    State(state): State<AppState>,
) -> Result<Response, SyncServerError> {
    Ok(ws.on_upgrade(move |socket| websocket_wrapped(state, socket, vault_id)))
}

async fn websocket_wrapped(state: AppState, stream: WebSocket, vault_id: VaultId) {
    info!("Websocket connection opened on vault '{vault_id}'");

    let result = websocket(state, stream, vault_id.clone()).await;

    if let Err(err) = result {
        error!("Websocket connection error on vault '{vault_id}': {err}");
    }

    warn!("Websocket connection closed on vault '{vault_id}'");
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebsocketHandshake {
    pub token: String,
    pub device_id: DeviceId,
    pub last_seen_vault_update_id: Option<VaultUpdateId>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WebsocketVaultUpdate {
    pub documents: Vec<DocumentVersionWithoutContent>,
    pub is_initial_sync: bool,
}

async fn websocket(
    state: AppState,
    stream: WebSocket,
    vault_id: VaultId,
) -> Result<(), SyncServerError> {
    let (mut sender, mut receiver) = stream.split();

    let handshake = if let Some(Ok(Message::Text(token))) = receiver.next().await {
        let handshake: WebsocketHandshake = serde_json::from_str(&token)
            .context("Failed to parse token")
            .map_err(server_error)?;

        auth(&state, handshake.token.trim(), &vault_id)?;

        handshake
    } else {
        return Err(unauthenticated_error(anyhow::anyhow!(
            "Failed to authenticate"
        )));
    };

    let mut rx = state.broadcasts.get_receiver(vault_id.clone()).await;

    let documents = if let Some(update_id) = handshake.last_seen_vault_update_id {
        state
            .database
            .get_latest_documents_since(&vault_id, update_id, None)
            .await
            .map_err(server_error)
    } else {
        state
            .database
            .get_latest_documents(&vault_id, None)
            .await
            .map_err(server_error)
    }?;

    send_update_over_websocket(
        &WebsocketVaultUpdate {
            documents,
            is_initial_sync: true,
        },
        &mut sender,
    )
    .await?;

    let mut send_task = tokio::spawn(async move {
        while let Ok(update) = rx.recv().await {
            if Some(&handshake.device_id) == update.origin_device_id.as_ref() {
                continue;
            }

            send_update_over_websocket(
                &WebsocketVaultUpdate {
                    documents: vec![update.document],
                    is_initial_sync: false,
                },
                &mut sender,
            )
            .await?;
        }

        Ok::<(), SyncServerError>(())
    });

    let mut recv_task =
        tokio::spawn(
            async move { while let Some(Ok(Message::Text(_text))) = receiver.next().await {} },
        );

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    };

    send_task
        .await
        .context("Websocket send task failed")
        .map_err(server_error)??;

    recv_task
        .await
        .context("Websocket receive task failed")
        .map_err(server_error)?;

    Ok(())
}

async fn send_update_over_websocket(
    update: &WebsocketVaultUpdate,
    sender: &mut SplitSink<WebSocket, Message>,
) -> Result<(), SyncServerError> {
    let serialized_update = serde_json::to_string(update)
        .context("Failed to serialize update")
        .map_err(server_error)?;

    sender
        .send(Message::Text(serialized_update))
        .await
        .context("Failed to send message over websocket")
        .map_err(server_error)
}
