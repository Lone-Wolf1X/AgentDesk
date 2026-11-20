//! OpenH264 software decoder wrapper.

use crate::error::EncodeError;
use crate::traits::{EncodedPacket, VideoDecoder};
use crate::yuv::yuv420p_to_bgra;
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
            let (w, h) = yuv.width_and_height();
            let bgra = yuv420p_to_bgra(yuv.y(), yuv.u(), yuv.v(), w, h);
            let stride = (w * 4) as u32;

            Ok(Frame {
                width: w as u32,
                height: h as u32,
                stride,
                bgra,
                timestamp: Instant::now(),
            })
        } else {
            // Some packets like parameter sets (SPS/PPS) don't immediately produce a picture
            Err(EncodeError::DecodingFailed(
                "Packet did not yield a decoded video picture (may be parameter set header)".to_string(),
            ))
        }
    }
}
