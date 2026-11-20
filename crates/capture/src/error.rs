//! Errors emitted by screen capture engines.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CaptureError {
    #[error("Wayland is not supported yet. Please log into an X11 session. Details: {0}")]
    WaylandUnsupported(String),

    #[error("Permission denied for {0}. Please check your system privacy settings.")]
    PermissionDenied(String),

    #[error("Display index {0} not found.")]
    DisplayNotFound(usize),

    #[error("Failed to initialize capture engine: {0}")]
    InitializationFailed(String),

    #[error("Frame capture failed: {0}")]
    CaptureFailed(String),

    #[error("Crop region [{crop_x}, {crop_y}, {crop_w}x{crop_h}] is out of bounds for frame [{frame_w}x{frame_h}]")]
    CropOutOfBounds {
        frame_w: u32,
        frame_h: u32,
        crop_x: u32,
        crop_y: u32,
        crop_w: u32,
        crop_h: u32,
    },

    #[error("System error: {0}")]
    Other(String),
}
