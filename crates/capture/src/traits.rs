//! ScreenCapturer trait definition.

use crate::error::CaptureError;
use agentdesk_core::{CaptureConfig, DisplayInfo, Frame, Rect};

/// Common trait for all screen capture implementations.
pub trait ScreenCapturer: Send + Sync {
    /// Captures a single frame from the display.
    fn capture_frame(&mut self) -> Result<Frame, CaptureError>;

    /// Returns metadata about the target display being captured.
    fn display_info(&self) -> Result<DisplayInfo, CaptureError>;

    /// Updates the capture configuration (FPS, target display, or crop region).
    fn update_config(&mut self, config: CaptureConfig) -> Result<(), CaptureError>;

    /// Returns the current configuration.
    fn config(&self) -> &CaptureConfig;
}

/// Helper function to apply crop region to a BGRA frame.
pub fn apply_crop(frame: Frame, crop: Rect) -> Result<Frame, CaptureError> {
    if crop.x + crop.width > frame.width || crop.y + crop.height > frame.height {
        return Err(CaptureError::CropOutOfBounds {
            frame_w: frame.width,
            frame_h: frame.height,
            crop_x: crop.x,
            crop_y: crop.y,
            crop_w: crop.width,
            crop_h: crop.height,
        });
    }

    let bpp = 4usize;
    let cropped_stride = crop.width * bpp as u32;
    let mut cropped_bgra = Vec::with_capacity((cropped_stride * crop.height) as usize);

    for row in 0..crop.height {
        let src_y = (crop.y + row) as usize;
        let src_start = (src_y * frame.stride as usize) + (crop.x as usize * bpp);
        let src_end = src_start + (crop.width as usize * bpp);
        cropped_bgra.extend_from_slice(&frame.bgra[src_start..src_end]);
    }

    Ok(Frame {
        width: crop.width,
        height: crop.height,
        stride: cropped_stride,
        bgra: cropped_bgra,
        timestamp: frame.timestamp,
    })
}
