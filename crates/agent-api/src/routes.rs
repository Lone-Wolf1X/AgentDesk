//! Request handlers for Agent API endpoints.

use crate::error::ApiError;
use crate::server::AppState;
use agentdesk_core::platform::detect_platform_environment;
use agentdesk_core::types::{MouseButton, ScreenSize};
use axum::extract::State;
use axum::Json;
use base64::Engine;
use image::{ImageBuffer, ImageFormat, Rgba};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use tracing::{debug, info};

/// Response body for `GET /health`.
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub platform: String,
    pub version: String,
}

/// Request body for `POST /screenshot`.
#[derive(Deserialize, Default)]
pub struct ScreenshotRequest {
    pub display: Option<usize>,
}

/// Response body for `POST /screenshot`.
#[derive(Serialize)]
pub struct ScreenshotResponse {
    pub width: u32,
    pub height: u32,
    pub png_base64: String,
}

/// Request body for `POST /click`.
#[derive(Deserialize)]
pub struct ClickRequest {
    pub x: f64,
    pub y: f64,
    #[serde(default = "default_button")]
    pub button: String,
    #[serde(default = "default_clicks")]
    pub clicks: u32,
}

fn default_button() -> String {
    "left".to_string()
}

fn default_clicks() -> u32 {
    1
}

/// Request body for `POST /move`.
#[derive(Deserialize)]
pub struct MoveRequest {
    pub x: f64,
    pub y: f64,
}

/// Request body for `POST /type`.
#[derive(Deserialize)]
pub struct TypeRequest {
    pub text: String,
}

/// Request body for `POST /key`.
#[derive(Deserialize)]
pub struct KeyRequest {
    pub combo: String,
}

/// Standard success response.
#[derive(Serialize)]
pub struct SuccessResponse {
    pub success: bool,
}

/// Handler: `GET /health`
pub async fn health_handler() -> Json<HealthResponse> {
    let env = detect_platform_environment();
    let platform = match env.kind {
        agentdesk_core::platform::PlatformKind::Linux => {
            if env.is_wayland {
                "linux-wayland"
            } else {
                "linux-x11"
            }
        }
        agentdesk_core::platform::PlatformKind::MacOS => "macos",
        agentdesk_core::platform::PlatformKind::Windows => "windows",
        agentdesk_core::platform::PlatformKind::Other => "other",
    };

    Json(HealthResponse {
        status: "ok".to_string(),
        platform: platform.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Handler: `GET /screen-size`
pub async fn screen_size_handler(
    State(state): State<AppState>,
) -> Result<Json<ScreenSize>, ApiError> {
    let capturer = state.capturer.lock().await;
    let info = capturer.display_info()?;
    Ok(Json(ScreenSize {
        width: info.width,
        height: info.height,
    }))
}

/// Handler: `POST /screenshot`
pub async fn screenshot_handler(
    State(state): State<AppState>,
    Json(_req): Json<ScreenshotRequest>,
) -> Result<Json<ScreenshotResponse>, ApiError> {
    let mut capturer = state.capturer.lock().await;
    let frame = capturer.capture_frame()?;

    // Convert to RGBA image buffer and encode to PNG
    let rgba_bytes = frame.to_rgba();
    let img_buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(frame.width, frame.height, rgba_bytes)
        .ok_or_else(|| {
            ApiError::ImageEncoding("Failed to create image buffer from raw pixels".to_string())
        })?;

    let mut png_bytes = Cursor::new(Vec::new());
    img_buffer
        .write_to(&mut png_bytes, ImageFormat::Png)
        .map_err(|e| ApiError::ImageEncoding(format!("PNG encoding failed: {e}")))?;

    let png_base64 = base64::engine::general_purpose::STANDARD.encode(png_bytes.into_inner());

    debug!("Captured screenshot: {}x{}", frame.width, frame.height);

    Ok(Json(ScreenshotResponse {
        width: frame.width,
        height: frame.height,
        png_base64,
    }))
}

/// Handler: `POST /move`
pub async fn move_handler(
    State(state): State<AppState>,
    Json(req): Json<MoveRequest>,
) -> Result<Json<SuccessResponse>, ApiError> {
    let mut injector = state.injector.lock().await;
    injector.move_mouse(req.x, req.y)?;
    Ok(Json(SuccessResponse { success: true }))
}

/// Handler: `POST /click`
pub async fn click_handler(
    State(state): State<AppState>,
    Json(req): Json<ClickRequest>,
) -> Result<Json<SuccessResponse>, ApiError> {
    let mut injector = state.injector.lock().await;

    // Move to position first
    injector.move_mouse(req.x, req.y)?;
    tokio::time::sleep(std::time::Duration::from_millis(15)).await;

    let button = match req.button.to_lowercase().as_str() {
        "left" => MouseButton::Left,
        "right" => MouseButton::Right,
        "middle" => MouseButton::Middle,
        _ => MouseButton::Left,
    };

    let clicks = req.clicks.max(1);
    for i in 0..clicks {
        injector.click_tap(button)?;
        if i + 1 < clicks {
            tokio::time::sleep(std::time::Duration::from_millis(80)).await;
        }
    }

    Ok(Json(SuccessResponse { success: true }))
}

/// Handler: `POST /type`
pub async fn type_handler(
    State(state): State<AppState>,
    Json(req): Json<TypeRequest>,
) -> Result<Json<SuccessResponse>, ApiError> {
    let mut injector = state.injector.lock().await;
    injector.type_text(&req.text)?;
    Ok(Json(SuccessResponse { success: true }))
}

/// Handler: `POST /key`
pub async fn key_handler(
    State(state): State<AppState>,
    Json(req): Json<KeyRequest>,
) -> Result<Json<SuccessResponse>, ApiError> {
    let mut injector = state.injector.lock().await;
    injector.simulate_combo(&req.combo)?;
    Ok(Json(SuccessResponse { success: true }))
}
