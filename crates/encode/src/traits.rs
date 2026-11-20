//! Video encoder and decoder traits.

use crate::error::EncodeError;
use agentdesk_core::Frame;
use serde::{Deserialize, Serialize};

/// An encoded video packet (e.g. H.264 NAL units).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncodedPacket {
    /// Compressed bitstream data.
    pub data: Vec<u8>,
    /// Whether this packet contains an IDR / keyframe.
    pub is_keyframe: bool,
    /// Capture / presentation timestamp in milliseconds.
    pub timestamp_ms: u64,
    /// Width of the video frame.
    pub width: u32,
    /// Height of the video frame.
    pub height: u32,
}

/// Abstract video encoder.
pub trait VideoEncoder: Send + Sync {
    /// Encodes a raw screen frame into compressed H.264 packets.
    fn encode_frame(&mut self, frame: &Frame) -> Result<EncodedPacket, EncodeError>;

    /// Requests that the next encoded frame be a keyframe (IDR).
    fn request_keyframe(&mut self);

    /// Updates target bitrate in kbps and target fps.
    fn update_bitrate(&mut self, bitrate_kbps: u32, fps: f32) -> Result<(), EncodeError>;
}

/// Abstract video decoder.
pub trait VideoDecoder: Send + Sync {
    /// Decodes an encoded packet back into raw BGRA / RGBA frame.
    fn decode_packet(&mut self, packet: &EncodedPacket) -> Result<Frame, EncodeError>;
}
