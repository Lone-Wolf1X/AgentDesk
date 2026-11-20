//! Synthetic / mock screen capturer for automated testing and CI runners.

use crate::error::CaptureError;
use crate::traits::{apply_crop, ScreenCapturer};
use agentdesk_core::{CaptureConfig, DisplayInfo, Frame};
use std::time::Instant;

/// Mock capturer that generates synthetic animated test pattern frames.
pub struct MockCapturer {
    config: CaptureConfig,
    width: u32,
    height: u32,
    frame_count: u64,
    start_time: Instant,
}

impl MockCapturer {
    pub fn new(width: u32, height: u32, config: CaptureConfig) -> Self {
        Self {
            config,
            width,
            height,
            frame_count: 0,
            start_time: Instant::now(),
        }
    }

    /// Convenience constructor with 1920x1080 default size.
    pub fn default_1080p() -> Self {
        Self::new(1920, 1080, CaptureConfig::default())
    }
}

impl ScreenCapturer for MockCapturer {
    fn capture_frame(&mut self) -> Result<Frame, CaptureError> {
        self.frame_count += 1;
        let stride = self.width * 4;
        let mut bgra = vec![0u8; (stride * self.height) as usize];

        // Generate synthetic colored test pattern with moving square
        let phase = (self.frame_count * 5) % self.width as u64;
        let sq_x = phase as u32;
        let sq_y = ((self.frame_count * 3) % self.height as u64) as u32;
        let sq_size = 50u32;

        for y in 0..self.height {
            for x in 0..self.width {
                let offset = (y * stride + x * 4) as usize;
                if x >= sq_x && x < sq_x + sq_size && y >= sq_y && y < sq_y + sq_size {
                    // White moving square
                    bgra[offset] = 255;
                    bgra[offset + 1] = 255;
                    bgra[offset + 2] = 255;
                    bgra[offset + 3] = 255;
                } else {
                    // Smooth gradient
                    bgra[offset] = ((x * 255) / self.width) as u8; // Blue
                    bgra[offset + 1] = ((y * 255) / self.height) as u8; // Green
                    bgra[offset + 2] = 120; // Red
                    bgra[offset + 3] = 255; // Alpha
                }
            }
        }

        let full_frame = Frame {
            width: self.width,
            height: self.height,
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

    fn display_info(&self) -> Result<DisplayInfo, CaptureError> {
        Ok(DisplayInfo {
            id: self.config.display_index,
            name: format!("Mock Display #{}", self.config.display_index),
            width: self.width,
            height: self.height,
            is_primary: true,
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
