// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.
// Proprietary and confidential.

//! Endpoint WebSocket para streaming de telemetría y configuración OTA (Downlink).

use crate::domain::state::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::{sink::SinkExt, stream::StreamExt};

// OTA Commands have been removed for security until mTLS/auth is implemented.

/// Endpoint para abrir la conexión WebSocket.
#[utoipa::path(
    get,
    path = "/api/ws",
    responses(
        (status = 101, description = "WebSocket upgrade exitoso")
    )
)]
#[allow(clippy::unused_async)]
pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Nos suscribimos al canal de broadcast de telemetría
    let mut rx_telemetry = state.tx_ws.subscribe();

    let mut send_task = tokio::spawn(async move {
        // 1. Enviar el estado actual del gemelo digital apenas se conecta
        {
            let twin = state.digital_twin.read().await;
            for (_, payload) in twin.iter() {
                if let Ok(json_str) = serde_json::to_string(payload) {
                    let _ = sender.send(Message::Text(json_str)).await;
                }
            }
        }

        // 2. Loop principal de streaming en vivo
        loop {
            match rx_telemetry.recv().await {
                Ok(payload) => {
                    if let Ok(json_str) = serde_json::to_string(&payload) {
                        if sender.send(Message::Text(json_str)).await.is_err() {
                            break; // Cliente desconectado
                        }
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                    tracing::warn!("WebSocket client lagged behind. Skipped {} messages.", skipped);
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    break; // Canal cerrado
                }
            }
        }
    });

    // Tarea 2: Leer del cliente WebSocket (Comandos OTA JSON)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    tracing::debug!("Mensaje ignorado (Read-Only WS): {}", text);
                }
                Message::Close(_) => {
                    tracing::debug!("Cliente WebSocket cerró la conexión.");
                    break;
                }
                _ => {}
            }
        }
    });

    // Si una de las dos tareas finaliza, abortamos la otra para limpiar la conexión.
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}
