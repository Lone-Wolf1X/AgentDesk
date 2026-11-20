//! Binary framing and high-level protocol messages for TCP transport.
//!
//! Packet structure:
//! `[u32 length (BE)][u8 msg_type][payload bytes...]`

use crate::error::CoreError;
use crate::types::{InputEvent, ScreenSize};
use bytes::{Buf, BufMut, BytesMut};
use serde::{Deserialize, Serialize};
use tokio_util::codec::{Decoder, Encoder};

/// Message type discriminant header byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MsgType {
    VideoFrame = 1,
    InputEvent = 2,
    Control = 3,
    Handshake = 4,
    Heartbeat = 5,
}

impl TryFrom<u8> for MsgType {
    type Error = CoreError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            1 => Ok(MsgType::VideoFrame),
            2 => Ok(MsgType::InputEvent),
            3 => Ok(MsgType::Control),
            4 => Ok(MsgType::Handshake),
            5 => Ok(MsgType::Heartbeat),
            other => Err(CoreError::InvalidMsgType(other)),
        }
    }
}

/// Client handshake request sent upon connection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandshakeRequest {
    pub token: String,
    pub client_version: String,
}

/// Host handshake response acknowledging authentication.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandshakeResponse {
    pub success: bool,
    pub reason: Option<String>,
    pub host_version: String,
    pub screen: ScreenSize,
}

/// Compressed video packet (e.g. H.264 NAL stream).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncodedVideoPacket {
    pub is_keyframe: bool,
    pub width: u32,
    pub height: u32,
    pub timestamp_ms: u64,
    pub data: Vec<u8>,
}

/// Bidirectional control messages.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", content = "payload")]
pub enum ControlMsg {
    Ping,
    Pong,
    RequestKeyframe,
    SetFps(u32),
    SetBitrate(u32),
    SessionLock { locked: bool },
}

/// Periodic heartbeat packet to keep connection alive and detect timeouts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeartbeatMsg {
    pub timestamp_ms: u64,
}

/// High-level strongly-typed message transferred over the network.
#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    VideoFrame(EncodedVideoPacket),
    Input(InputEvent),
    Control(ControlMsg),
    HandshakeReq(HandshakeRequest),
    HandshakeResp(HandshakeResponse),
    Heartbeat(HeartbeatMsg),
}

/// Tokio-util FramingCodec implementing `[u32 len][u8 msg_type][payload]`.
#[derive(Default)]
pub struct FramingCodec {
    max_payload_size: usize,
}

impl FramingCodec {
    /// Creates a codec with the default maximum payload size (32MB for high-res frames).
    pub fn new() -> Self {
        Self {
            max_payload_size: 32 * 1024 * 1024,
        }
    }

    /// Sets custom maximum payload size limit.
    pub fn with_max_payload(mut self, max: usize) -> Self {
        self.max_payload_size = max;
        self
    }
}

impl Decoder for FramingCodec {
    type Item = (MsgType, Vec<u8>);
    type Error = CoreError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Need at least 4 bytes (len) + 1 byte (msg_type) = 5 bytes
        if src.len() < 5 {
            return Ok(None);
        }

        // Read length prefix without advancing cursor
        let length = u32::from_be_bytes([src[0], src[1], src[2], src[3]]) as usize;

        if length > self.max_payload_size {
            return Err(CoreError::PayloadTooLarge {
                size: length,
                max: self.max_payload_size,
            });
        }

        let total_frame_len = 4 + 1 + length;
        if src.len() < total_frame_len {
            // Wait for more bytes to arrive
            src.reserve(total_frame_len - src.len());
            return Ok(None);
        }

        // Discard the 4-byte length prefix
        src.advance(4);

        // Read 1-byte message type
        let msg_type_raw = src.get_u8();
        let msg_type = MsgType::try_from(msg_type_raw)?;

        // Read payload bytes
        let payload = src.split_to(length).to_vec();

        Ok(Some((msg_type, payload)))
    }
}

impl Encoder<(MsgType, Vec<u8>)> for FramingCodec {
    type Error = CoreError;

    fn encode(&mut self, item: (MsgType, Vec<u8>), dst: &mut BytesMut) -> Result<(), Self::Error> {
        let (msg_type, payload) = item;
        let length = payload.len();

        if length > self.max_payload_size {
            return Err(CoreError::PayloadTooLarge {
                size: length,
                max: self.max_payload_size,
            });
        }

        dst.reserve(4 + 1 + length);
        dst.put_u32(length as u32);
        dst.put_u8(msg_type as u8);
        dst.put_slice(&payload);

        Ok(())
    }
}
