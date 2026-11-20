//! API error types with Axum IntoResponse implementation.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Capture error: {0}")]
    Capture(#[from] agentdesk_capture::CaptureError),

    #[error("Input injection error: {0}")]
    Input(#[from] agentdesk_input::InputError),

    #[error("Image encoding error: {0}")]
    ImageEncoding(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Capture(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            ApiError::Input(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            ApiError::ImageEncoding(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}
