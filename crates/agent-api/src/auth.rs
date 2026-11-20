//! Bearer token authentication middleware for Agent API routes.

use crate::error::ApiError;
use agentdesk_core::security::constant_time_token_match;
use axum::extract::Request;
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::Response;

/// Middleware to enforce `Authorization: Bearer <token>` on all agent endpoints.
pub async fn require_auth(
    expected_token: String,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            ApiError::Unauthorized(
                "Missing 'Authorization: Bearer <token>' header in request".to_string(),
            )
        })?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .map(|s| s.trim())
        .ok_or_else(|| {
            ApiError::Unauthorized(
                "Invalid Authorization format. Expected: 'Bearer <token>'".to_string(),
            )
        })?;

    if !constant_time_token_match(&expected_token, token) {
        return Err(ApiError::Unauthorized("Invalid API bearer token".to_string()));
    }

    Ok(next.run(req).await)
}
