//! Core library for AgentDesk: types, network protocol, configuration,
//! security primitives, and platform detection.

pub mod config;
pub mod error;
pub mod platform;
pub mod protocol;
pub mod security;
pub mod types;

pub use config::{AgentApiConfig, CaptureConfig, HostConfig, ViewerConfig};
pub use error::CoreError;
pub use platform::{detect_platform_environment, PlatformEnvironment, PlatformKind};
pub use protocol::{
    ControlMsg, EncodedVideoPacket, FramingCodec, HandshakeRequest, HandshakeResponse,
    HeartbeatMsg, Message, MsgType,
};
pub use security::{constant_time_token_match, generate_self_signed_cert, CertInfo};
pub use types::{
    DisplayInfo, Frame, InputEvent, Key, KeyCombo, MouseButton, Rect, ScreenSize,
};
