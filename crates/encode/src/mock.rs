//! Mock video encoder and decoder for tests and headless environments.

use crate::error::EncodeError;
use crate::traits::{EncodedPacket, VideoDecoder, VideoEncoder};
use agentdesk_core::Frame;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Mock encoder that writes uncompressed / lightly framed test packets.
pub struct MockEncoder {
    width: u32,
    height: u32,
    force_keyframe: bool,
}

impl MockEncoder {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            force_keyframe: true,
        }
    }
}

impl VideoEncoder for MockEncoder {
    fn encode_frame(&mut self, frame: &Frame) -> Result<EncodedPacket, EncodeError> {
        let is_keyframe = self.force_keyframe;
        self.force_keyframe = false;

        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // In mock mode, we pass a copy of the pixel data or a lightweight representation
        Ok(EncodedPacket {
            data: frame.bgra.clone(),
            is_keyframe,
            timestamp_ms,
            width: frame.width,
            height: frame.height,
        })
    }

    fn request_keyframe(&mut self) {
        self.force_keyframe = true;
    }

    fn update_bitrate(&mut self, _bitrate_kbps: u32, _fps: f32) -> Result<(), EncodeError> {
        Ok(())
    }
}

/// Mock decoder matching `MockEncoder`.
#[derive(Default)]
pub struct MockDecoder;

impl MockDecoder {
    pub fn new() -> Self {
        Self
    }
}

impl VideoDecoder for MockDecoder {
    fn decode_packet(&mut self, packet: &EncodedPacket) -> Result<Frame, EncodeError> {
        Ok(Frame {
            width: packet.width,
            height: packet.height,
            stride: packet.width * 4,
            bgra: packet.data.clone(),
            timestamp: Instant::now(),
        })
    }
}
