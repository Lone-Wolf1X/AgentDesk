//! WebRTC Transport Stub for Phase 2 integration.
//!
//! Provides a drop-in architectural stub for str0m or webrtc-rs data channels and media tracks.

use crate::error::NetError;
use crate::traits::Transport;
use agentdesk_core::protocol::EncodedVideoPacket;
use agentdesk_core::types::InputEvent;
use async_trait::async_trait;

/// WebRTC transport implementation stub for Phase 2.
pub struct WebRtcTransportStub {
    room_id: String,
    signal_server_url: String,
}

impl WebRtcTransportStub {
    pub fn new(room_id: String, signal_server_url: String) -> Self {
        Self {
            room_id,
            signal_server_url,
        }
    }
}

#[async_trait]
impl Transport for WebRtcTransportStub {
    async fn connect(&mut self) -> Result<(), NetError> {
        Err(NetError::Transport(format!(
            "WebRTC transport is planned for Phase 2. Connect to room '{}' via '{}' will be activated with str0m.",
            self.room_id, self.signal_server_url
        )))
    }

    async fn send_video_frame(&mut self, _packet: EncodedVideoPacket) -> Result<(), NetError> {
        Err(NetError::Transport("WebRTC Phase 2 stub".to_string()))
    }

    async fn recv_video_frame(&mut self) -> Result<EncodedVideoPacket, NetError> {
        Err(NetError::Transport("WebRTC Phase 2 stub".to_string()))
    }

    async fn send_input_event(&mut self, _event: InputEvent) -> Result<(), NetError> {
        Err(NetError::Transport("WebRTC Phase 2 stub".to_string()))
    }

    async fn close(&mut self) -> Result<(), NetError> {
        Ok(())
    }
}
