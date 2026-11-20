// AgentDesk — Free & Open Source Remote Desktop
// Copyright (c) 2025 Next Gen Innovations Private Limited
// Developer: Abhishek Kumar Paswan
// Website: https://nextgeninnovations.com.np/
// License: MIT — see LICENSE file
// Made in Nepal 🇳🇵

use crate::types::Rect;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Host server configuration options.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HostConfig {
    /// Address to bind TCP server for viewers (default "0.0.0.0:9090").
    pub listen_addr: SocketAddr,
    /// Secret authentication password / token.
    pub token: String,
    /// Target frame rate per second.
    pub fps: u32,
    /// Target video bitrate in kbps.
    pub bitrate_kbps: u32,
    /// Target display index to capture.
    pub display_index: usize,
    /// Whether TLS is disabled (plain TCP for local testing).
    pub disable_tls: bool,
    /// Custom TLS certificate path.
    pub tls_cert_path: Option<String>,
    /// Custom TLS private key path.
    pub tls_key_path: Option<String>,
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:9090".parse().expect("valid listen addr"),
            token: "pass123".to_string(),
            fps: 20,
            bitrate_kbps: 2000,
            display_index: 0,
            disable_tls: false,
            tls_cert_path: None,
            tls_key_path: None,
        }
    }
}

/// Viewer client configuration options.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ViewerConfig {
    /// Remote host address to connect to.
    pub connect_addr: SocketAddr,
    /// Authentication password / token.
    pub token: String,
    /// Expected SHA-256 TLS cert fingerprint for pinning.
    pub tls_fingerprint: Option<String>,
    /// Whether TLS is disabled (matches host).
    pub disable_tls: bool,
    /// Directory to save decoded frames as images.
    pub save_frames_dir: Option<String>,
    /// Exit after receiving N frames.
    pub max_frames: Option<u64>,
}

impl Default for ViewerConfig {
    fn default() -> Self {
        Self {
            connect_addr: "127.0.0.1:9090".parse().expect("valid connect addr"),
            token: "pass123".to_string(),
            tls_fingerprint: None,
            disable_tls: false,
            save_frames_dir: None,
            max_frames: None,
        }
    }
}

/// Screen capturer runtime configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CaptureConfig {
    /// Target FPS.
    pub fps: u32,
    /// Target display index.
    pub display_index: usize,
    /// Optional region crop rectangle.
    pub crop_region: Option<Rect>,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            fps: 20,
            display_index: 0,
            crop_region: None,
        }
    }
}
