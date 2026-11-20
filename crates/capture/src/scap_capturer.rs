//! Production screen capturer utilizing `scap` cross-platform APIs.

use crate::error::CaptureError;
use crate::preflight::check_capture_permissions;
use crate::traits::{apply_crop, ScreenCapturer};
use agentdesk_core::{CaptureConfig, DisplayInfo, Frame};
use std::time::Instant;
use tracing::{debug, info, warn};

/// Production Screen Capturer backed by `scap`.
pub struct ScapCapturer {
    config: CaptureConfig,
    capturer: Option<scap::capturer::Capturer>,
    width: u32,
    height: u32,
    display_name: String,
}

impl ScapCapturer {
    /// Creates and initializes a new ScapCapturer after verifying platform permissions.
    pub fn new(config: CaptureConfig) -> Result<Self, CaptureError> {
        // Step 1: Preflight checks (Wayland, macOS permissions, etc.)
        check_capture_permissions()?;

        // Step 2: Initialize scap
        let mut capturer = scap::capturer::Capturer::build(scap::capturer::Options {
            fps: config.fps,
            show_cursor: true,
            show_highlight: false,
            target: None, // Primary display
            ..Default::default()
        })
        .map_err(|e| CaptureError::InitializationFailed(format!("{e:?}")))?;

        capturer.start_capture();

        let width = 1920; // Will be updated on first frame
        let height = 1080;

        Ok(Self {
            config,
            capturer: Some(capturer),
            width,
            height,
            display_name: "Primary Display".to_string(),
        })
    }
}

impl ScreenCapturer for ScapCapturer {
    fn capture_frame(&mut self) -> Result<Frame, CaptureError> {
        let capturer = self
            .capturer
            .as_mut()
            .ok_or_else(|| CaptureError::CaptureFailed("Capturer not initialized".to_string()))?;

        match capturer.get_next_frame() {
            Ok(scap_frame) => {
                match scap_frame {
                    scap::frame::Frame::Video(video_frame) => {
                        let (w, h, bgra, stride) = match video_frame {
                            scap::frame::VideoFrame::Bgra(f) => {
                                (f.width as u32, f.height as u32, f.raw, (f.width * 4) as u32)
                            }
                            scap::frame::VideoFrame::Rgba(f) => {
                                // Convert RGBA to BGRA
                                let mut raw = f.raw;
                                for chunk in raw.chunks_exact_mut(4) {
                                    let r = chunk[0];
                                    let b = chunk[2];
                                    chunk[0] = b;
                                    chunk[2] = r;
                                }
                                (f.width as u32, f.height as u32, raw, (f.width * 4) as u32)
                            }
                            other => {
                                return Err(CaptureError::CaptureFailed(format!(
                                    "Unsupported video frame format: {other:?}"
                                )));
                            }
                        };

                        self.width = w;
                        self.height = h;

                        let full_frame = Frame {
                            width: w,
                            height: h,
                            stride,
                            bgra,
                            timestamp: Instant::now(),
                        };

                        if let Some(crop) = self.config.crop_region {
                            apply_crop(full_frame, crop)
                        } else {
                            Ok(full_frame)
                        }
                    }
                    _ => Err(CaptureError::CaptureFailed("Non-video frame received".to_string())),
                }
            }
            Err(e) => {
                // Check if macOS permission was revoked
                let err_str = format!("{e:?}");
                if err_str.to_lowercase().contains("permission") {
                    Err(CaptureError::PermissionDenied("Screen Recording".to_string()))
                } else {
                    Err(CaptureError::CaptureFailed(err_str))
                }
            }
        }
    }

    fn display_info(&self) -> Result<DisplayInfo, CaptureError> {
        Ok(DisplayInfo {
            id: self.config.display_index,
            name: self.display_name.clone(),
            width: self.width,
            height: self.height,
            is_primary: self.config.display_index == 0,
            scale_factor: 1,
        })
    }

    fn update_config(&mut self, config: CaptureConfig) -> Result<(), CaptureError> {
        self.config = config;
        Ok(())
    }

    fn config(&self) -> &CaptureConfig {
        &self.config
    }
}

impl Drop for ScapCapturer {
    fn drop(&mut self) {
        if let Some(mut capturer) = self.capturer.take() {
            capturer.stop_capture();
        }
    }
}
