// AgentDesk — Free & Open Source Remote Desktop
// Copyright (c) 2025 Next Gen Innovations Private Limited
// Developer: Abhishek Kumar Paswan
// Website: https://nextgeninnovations.com.np/
// License: MIT — see LICENSE file
// Made in Nepal 🇳🇵

pub mod config;
pub mod error;
pub mod network_info;
pub mod platform;
pub mod protocol;
pub mod security;
pub mod types;

pub use config::{CaptureConfig, HostConfig, ViewerConfig};
pub use error::CoreError;
pub use network_info::{get_all_local_ips, get_local_ip};
pub use platform::{detect_platform_environment, PlatformEnvironment, PlatformKind};
pub use protocol::{
    ControlMsg, EncodedVideoPacket, FramingCodec, HandshakeRequest, HandshakeResponse,
    HeartbeatMsg, Message, MsgType,
};
pub use security::{constant_time_token_match, generate_self_signed_cert, CertInfo};
pub use types::{
    DisplayInfo, Frame, InputEvent, Key, KeyCombo, MouseButton, Rect, ScreenSize,
};
