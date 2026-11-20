//! Core error types.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Invalid message type discriminant: {0}")]
    InvalidMsgType(u8),

    #[error("Payload too large: received {size} bytes, maximum allowed is {max} bytes")]
    PayloadTooLarge { size: usize, max: usize },

    #[error("Serialization / deserialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Security or cryptographic error: {0}")]
    SecurityError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}
