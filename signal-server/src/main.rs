// AgentDesk — Free & Open Source Remote Desktop
// Copyright (c) 2025 Next Gen Innovations Private Limited
// Developer: Abhishek Kumar Paswan
// Website: https://nextgeninnovations.com.np/
// License: MIT — see LICENSE file
// Made in Nepal 🇳🇵

use anyhow::Result;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use clap::Parser;
use futures::StreamExt;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::info;

#[derive(Parser, Debug)]
#[command(
    name = "agentdesk-signal",
    version = "0.1.0",
    about = "AgentDesk Signaling Server — 9-Digit ID Brokering — Made in Nepal 🇳🇵\n(c) 2025 Next Gen Innovations Private Limited — https://nextgeninnovations.com.np/\nDeveloper: Abhishek Kumar Paswan"
)]
struct Cli {
    /// Address to bind WebSocket signaling server
    #[arg(short, long, default_value = "0.0.0.0:8080")]
    listen: SocketAddr,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HostRecord {
    pub device_id: String,
    pub ip_endpoint: String,
    pub last_seen_secs: u64,
}

#[derive(Clone, Default)]
pub struct AppState {
    pub hosts: Arc<RwLock<HashMap<String, HostRecord>>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    println!(
        "\n=======================================================================\n\
         🇳🇵 AgentDesk Signaling & ID Server v0.1.0\n\
         Free & Open Source Remote Desktop — Made in Nepal 🇳🇵\n\
         Copyright (c) 2025 Next Gen Innovations Private Limited\n\
         Website: https://nextgeninnovations.com.np/\n\
         Developer: Abhishek Kumar Paswan\n\
         =======================================================================\n"
    );

    let state = AppState::default();

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/register", post(register_host))
        .route("/resolve/:id", get(resolve_id))
        .route("/ws/:id", get(ws_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(cli.listen).await?;
    info!("Signaling Server listening on http://{}", cli.listen);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_check() -> &'static str {
    "AgentDesk Signal Server: Healthy (Next Gen Innovations Private Limited 🇳🇵)"
}

#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
    pub device_id: Option<String>,
    pub ip_endpoint: String,
}

#[derive(Serialize, Debug)]
pub struct RegisterResponse {
    pub device_id: String,
    pub status: String,
}

async fn register_host(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    let device_id = match payload.device_id {
        Some(id) => id,
        None => {
            let num: u32 = rand::thread_rng().gen_range(100_000_000..=999_999_999);
            format!("{num}")
        }
    };

    let record = HostRecord {
        device_id: device_id.clone(),
        ip_endpoint: payload.ip_endpoint,
        last_seen_secs: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };

    let mut hosts = state.hosts.write().await;
    hosts.insert(device_id.clone(), record);
    info!("Registered host with 9-digit ID: {device_id}");

    Ok(Json(RegisterResponse {
        device_id,
        status: "registered".to_string(),
    }))
}

async fn resolve_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let hosts = state.hosts.read().await;
    if let Some(record) = hosts.get(&id) {
        Json(record.clone()).into_response()
    } else {
        (StatusCode::NOT_FOUND, "ID not found or host offline").into_response()
    }
}

async fn ws_handler(
    Path(id): Path<String>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, id))
}

async fn handle_socket(mut socket: WebSocket, id: String) {
    info!("Signaling WebSocket channel connected for ID: {id}");
    while let Some(Ok(msg)) = socket.next().await {
        if let Message::Text(text) = msg {
            info!("Signal message on channel {id}: {text}");
        }
    }
}
