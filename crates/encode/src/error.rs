//! Error types for video encoding and decoding.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum EncodeError {
    #[error("Failed to initialize OpenH264 encoder: {0}")]
    EncoderInitFailed(String),

    #[error("Failed to encode frame: {0}")]
    EncodingFailed(String),

    #[error("Failed to initialize OpenH264 decoder: {0}")]
    DecoderInitFailed(String),

    #[error("Failed to decode video packet: {0}")]
    DecodingFailed(String),

    #[error("Hardware encoder '{0}' not supported or disabled via feature flags")]
    HardwareNotSupported(String),

    #[error("Color conversion error: {0}")]
    ColorConversion(String),

    #[error("Encoder error: {0}")]
    Other(String),
}
