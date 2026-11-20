//! OpenH264 software decoder wrapper.

use crate::error::EncodeError;
use crate::traits::{EncodedPacket, VideoDecoder};
use agentdesk_core::Frame;
use openh264::decoder::Decoder;
use openh264::formats::YUVSource;
use std::time::Instant;

/// OpenH264 decoder for receiving viewer.
pub struct OpenH264Decoder {
    decoder: Decoder,
}

impl OpenH264Decoder {
    /// Creates a new OpenH264 decoder instance.
    pub fn new() -> Result<Self, EncodeError> {
        let decoder =
            Decoder::new().map_err(|e| EncodeError::DecoderInitFailed(format!("{e:?}")))?;
        Ok(Self { decoder })
    }
}

impl VideoDecoder for OpenH264Decoder {
    fn decode_packet(&mut self, packet: &EncodedPacket) -> Result<Frame, EncodeError> {
        let decoded_yuv = self
            .decoder
            .decode(&packet.data)
            .map_err(|e| EncodeError::DecodingFailed(format!("{e:?}")))?;

        if let Some(yuv) = decoded_yuv {
            let (w, h) = yuv.dimensions();
            let mut rgba = vec![0u8; w * h * 4];
            yuv.write_rgba8(&mut rgba);

            // Convert RGBA to BGRA
            let mut bgra = rgba;
            for chunk in bgra.chunks_exact_mut(4) {
                let r = chunk[0];
                let b = chunk[2];
                chunk[0] = b;
                chunk[2] = r;
            }

            let stride = (w * 4) as u32;

            Ok(Frame {
                width: w as u32,
                height: h as u32,
                stride,
                bgra,
                timestamp: Instant::now(),
            })
        } else {
            Err(EncodeError::DecodingFailed(
                "Packet did not yield a decoded video picture (may be parameter set header)".to_string(),
            ))
        }
    }
}
