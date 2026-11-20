// AgentDesk — Free & Open Source Remote Desktop
// Copyright (c) 2025 Next Gen Innovations Private Limited
// Developer: Abhishek Kumar Paswan
// Website: https://nextgeninnovations.com.np/
// License: MIT — see LICENSE file
// Made in Nepal 🇳🇵

use egui::TextureHandle;
use std::net::{IpAddr, SocketAddr};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct SessionStats {
    pub frames_received: u64,
    pub bytes_received: u64,
    pub current_fps: f32,
    pub latency_ms: u32,
}

impl Default for SessionStats {
    fn default() -> Self {
        Self {
            frames_received: 0,
            bytes_received: 0,
            current_fps: 0.0,
            latency_ms: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IncomingRequest {
    pub viewer_addr: SocketAddr,
    pub viewer_id: Option<String>,
    pub password_ok: bool,
    pub tls_fingerprint: String,
}

pub enum AppState {
    /// Showing "This Device" + "Connect to Remote" UI
    Idle {
        local_ip: Option<IpAddr>,
        local_port: u16,
        my_id: Option<String>,
        password: String,
        host_listening: bool,
        incoming_request: Option<IncomingRequest>,
    },
    /// Currently hosting a session (remote viewer active)
    Hosting {
        viewer_id: String,
        connected_since: Instant,
        stats: SessionStats,
    },
    /// Connecting to remote in progress
    Connecting {
        target: String,
        error: Option<String>,
    },
    /// Currently viewing a remote session
    Viewing {
        remote_addr: String,
        texture: Option<TextureHandle>,
        connected_since: Instant,
        stats: SessionStats,
    },
}
