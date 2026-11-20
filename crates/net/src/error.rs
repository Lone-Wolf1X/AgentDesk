//! Network transport error definitions.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Handshake failed: {0}")]
    HandshakeFailed(String),

    #[error("TLS configuration or certificate error: {0}")]
    TlsError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Core protocol error: {0}")]
    Core(#[from] agentdesk_core::error::CoreError),

    #[error("Connection closed unexpectedly")]
    ConnectionClosed,

    #[error("Transport error: {0}")]
    Transport(String),
}
