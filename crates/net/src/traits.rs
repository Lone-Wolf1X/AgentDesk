//! Transport abstraction for Phase 2 WebRTC integration.

use crate::error::NetError;
use agentdesk_core::protocol::EncodedVideoPacket;
use agentdesk_core::types::InputEvent;
use async_trait::async_trait;

/// Abstract transport trait enabling interchangeable TCP or WebRTC transport.
#[async_trait]
pub trait Transport: Send + Sync {
    /// Connects or initializes the transport connection.
    async fn connect(&mut self) -> Result<(), NetError>;

    /// Sends an encoded video packet to the remote peer.
    async fn send_video_frame(&mut self, packet: EncodedVideoPacket) -> Result<(), NetError>;

    /// Receives the next video frame from the remote peer.
    async fn recv_video_frame(&mut self) -> Result<EncodedVideoPacket, NetError>;

    /// Sends an input event to the remote host.
    async fn send_input_event(&mut self, event: InputEvent) -> Result<(), NetError>;

    /// Closes the transport connection gracefully.
    async fn close(&mut self) -> Result<(), NetError>;
}
