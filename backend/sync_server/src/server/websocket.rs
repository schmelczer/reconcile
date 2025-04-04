use anyhow::Context;
use axum::{
    extract::{
        Path, Query, State,
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
use serde::Deserialize;

use super::auth::auth;
use crate::{
    app_state::{
        AppState,
        database::models::{DocumentVersionWithoutContent, VaultId, VaultUpdateId},
    },
    errors::{SyncServerError, server_error, unauthenticated_error},
};

// This is required for aide to infer the path parameter types and names
#[derive(Deserialize, JsonSchema)]
pub struct WebsocketPathParams {
    vault_id: VaultId,
}

// This is required for aide to infer the path parameter types and names
#[derive(Deserialize, JsonSchema)]
pub struct QueryParams {
    since_update_id: Option<VaultUpdateId>,
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path(WebsocketPathParams { vault_id }): Path<WebsocketPathParams>,
    Query(QueryParams { since_update_id }): Query<QueryParams>,
    State(state): State<AppState>,
) -> Result<Response, SyncServerError> {
    Ok(ws.on_upgrade(move |socket| websocket_wrapped(state, socket, vault_id, since_update_id)))
}

async fn websocket_wrapped(
    state: AppState,
    stream: WebSocket,
    vault_id: VaultId,
    since_update_id: Option<VaultUpdateId>,
) {
    info!("Websocket connection opened on vault '{vault_id}'");

    let result = websocket(state, stream, vault_id.clone(), since_update_id).await;

    if let Err(err) = result {
        error!("Websocket connection error on vault '{vault_id}': {err}");
    }

    warn!("Websocket connection closed on vault '{vault_id}'");
}

async fn websocket(
    state: AppState,
    stream: WebSocket,
    vault_id: VaultId,
    since_update_id: Option<VaultUpdateId>,
) -> Result<(), SyncServerError> {
    let (mut sender, mut receiver) = stream.split();

    if let Some(Ok(Message::Text(token))) = receiver.next().await {
        auth(&state, &token, &vault_id)?;
    } else {
        return Err(unauthenticated_error(anyhow::anyhow!(
            "Failed to authenticate"
        )));
    }

    let mut rx = state.broadcasts.get_receiver(vault_id.clone()).await;

    let documents = if let Some(since_update_id) = since_update_id {
        state
            .database
            .get_latest_documents_since(&vault_id, since_update_id, None)
            .await
            .map_err(server_error)
    } else {
        state
            .database
            .get_latest_documents(&vault_id, None)
            .await
            .map_err(server_error)
    }?;

    for document in documents {
        send_document_over_websocket(document, &mut sender).await?;
    }

    let mut send_task = tokio::spawn(async move {
        while let Ok(update) = rx.recv().await {
            send_document_over_websocket(update, &mut sender).await?;
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

async fn send_document_over_websocket(
    document: DocumentVersionWithoutContent,
    sender: &mut SplitSink<WebSocket, Message>,
) -> Result<(), SyncServerError> {
    let serialized_update = serde_json::to_string(&document)
        .context("Failed to serialize update")
        .map_err(server_error)?;

    sender
        .send(Message::Text(serialized_update))
        .await
        .context("Failed to send message over websocket")
        .map_err(server_error)
}
