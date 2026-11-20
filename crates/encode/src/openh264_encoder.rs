//! OpenH264 software encoder wrapper.

use crate::error::EncodeError;
use crate::traits::{EncodedPacket, VideoEncoder};
use crate::yuv::bgra_to_yuv420p;
use agentdesk_core::Frame;
use openh264::encoder::Encoder;
use openh264::formats::YUVSource;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info};

/// Structure representing planar YUV slice source for OpenH264.
struct YuvBufferSource<'a> {
    width: usize,
    height: usize,
    y: &'a [u8],
    u: &'a [u8],
    v: &'a [u8],
}

impl<'a> YUVSource for YuvBufferSource<'a> {
    fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn strides(&self) -> (usize, usize, usize) {
        (self.width, (self.width + 1) / 2, (self.width + 1) / 2)
    }

    fn y(&self) -> &[u8] {
        self.y
    }

    fn u(&self) -> &[u8] {
        self.u
    }

    fn v(&self) -> &[u8] {
        self.v
    }
}

/// Software H.264 video encoder powered by OpenH264.
pub struct OpenH264Encoder {
    encoder: Option<Encoder>,
    width: u32,
    height: u32,
    bitrate_kbps: u32,
    fps: f32,
    force_keyframe: bool,
}

impl OpenH264Encoder {
    /// Creates a new OpenH264 encoder for the given dimensions and bitrate.
    pub fn new(width: u32, height: u32, bitrate_kbps: u32, fps: f32) -> Result<Self, EncodeError> {
        let mut encoder = Encoder::new()
            .map_err(|e| EncodeError::EncoderInitFailed(format!("{e:?}")))?;

        encoder.force_intra_frame();

        info!(
            "Initialized OpenH264 encoder: {width}x{height} @ {fps}fps, bitrate: {bitrate_kbps}kbps"
        );

        Ok(Self {
            encoder: Some(encoder),
            width,
            height,
            bitrate_kbps,
            fps,
            force_keyframe: true,
        })
    }

    /// Re-initializes encoder when resolution changes.
    fn recreate_encoder(&mut self, new_w: u32, new_h: u32) -> Result<(), EncodeError> {
        info!("Recreating OpenH264 encoder for new resolution: {new_w}x{new_h}");
        let mut encoder = Encoder::new()
            .map_err(|e| EncodeError::EncoderInitFailed(format!("{e:?}")))?;

        encoder.force_intra_frame();

        self.encoder = Some(encoder);
        self.width = new_w;
        self.height = new_h;
        self.force_keyframe = true;
        Ok(())
    }
}

impl VideoEncoder for OpenH264Encoder {
    fn encode_frame(&mut self, frame: &Frame) -> Result<EncodedPacket, EncodeError> {
        // Handle dynamic resolution changes
        if frame.width != self.width || frame.height != self.height {
            self.recreate_encoder(frame.width, frame.height)?;
        }

        let (y, u, v) = bgra_to_yuv420p(
            &frame.bgra,
            frame.width as usize,
            frame.height as usize,
            frame.stride as usize,
        );

        let yuv_source = YuvBufferSource {
            width: frame.width as usize,
            height: frame.height as usize,
            y: &y,
            u: &u,
            v: &v,
        };

        let encoder = self
            .encoder
            .as_mut()
            .ok_or_else(|| EncodeError::EncodingFailed("Encoder not initialized".to_string()))?;

        let is_keyframe = self.force_keyframe;
        if self.force_keyframe {
            encoder.force_intra_frame();
            self.force_keyframe = false;
        }

        let stream = encoder
            .encode(&yuv_source)
            .map_err(|e| EncodeError::EncodingFailed(format!("{e:?}")))?;

        let data = stream.to_vec();

        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Ok(EncodedPacket {
            data,
            is_keyframe,
            timestamp_ms,
            width: frame.width,
            height: frame.height,
        })
    }

    fn request_keyframe(&mut self) {
        debug!("Keyframe requested for next frame");
        self.force_keyframe = true;
    }

    fn update_bitrate(&mut self, bitrate_kbps: u32, fps: f32) -> Result<(), EncodeError> {
        self.bitrate_kbps = bitrate_kbps;
        self.fps = fps;
        self.recreate_encoder(self.width, self.height)
    }
}
