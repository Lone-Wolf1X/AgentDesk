//! WebSocket Signaling Server for WebRTC SDP / ICE candidate exchange (`agentdesk-signal`).

use anyhow::Result;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use clap::Parser;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::CorsLayer;
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(name = "agentdesk-signal", author, version, about = "AgentDesk WebRTC Signaling Server")]
struct Cli {
    /// Address to bind WebSocket signaling server
    #[arg(short, long, default_value = "0.0.0.0:8080")]
    listen: SocketAddr,
}

/// WebRTC Signaling message protocol.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
enum SignalMessage {
    Join { client_id: String },
    Offer { sdp: String },
    Answer { sdp: String },
    Candidate { candidate: String, sdp_mid: Option<String>, sdp_mline_index: Option<u16> },
    Leave,
    Error { message: String },
}

#[derive(Clone, Default)]
struct AppState {
    rooms: Arc<RwLock<HashMap<String, broadcast::Sender<String>>>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    info!("Starting AgentDesk WebRTC Signaling Server on {}", cli.listen);
    let state = AppState::default();

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/ws/:room_id", get(ws_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(cli.listen).await?;
    info!("Signaling Server listening on http://{}", cli.listen);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_check() -> &'static str {
    "agentdesk-signal: healthy"
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
    State(state): State<AppState>,
) -> IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, room_id, state))
}

async fn handle_socket(socket: WebSocket, room_id: String, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Get or create broadcast channel for this room
    let tx = {
        let mut rooms = state.rooms.write().await;
        rooms
            .entry(room_id.clone())
            .or_insert_with(|| broadcast::channel(64).0)
            .clone()
    };

    let mut rx = tx.subscribe();

    info!("Client joined signaling room: '{room_id}'");

    // Task to forward broadcast messages to this client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Task to receive messages from this client and broadcast to room
    let tx_clone = tx.clone();
    let room_clone = room_id.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                let _ = tx_clone.send(text);
            }
        }
        info!("Client left signaling room: '{room_clone}'");
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
}
