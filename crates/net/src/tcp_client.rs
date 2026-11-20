//! TCP Viewer Client: connects to host, authenticates, receives video stream,
//! and transmits input events.

use crate::error::NetError;
use crate::tls::create_client_tls_config;
use agentdesk_core::protocol::{
    EncodedVideoPacket, FramingCodec, HandshakeRequest, HandshakeResponse, MsgType,
    SessionPermissions,
};
use agentdesk_core::types::InputEvent;
use futures::{SinkExt, StreamExt};
use rustls::pki_types::ServerName;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_util::codec::Framed;
use tracing::{info, warn};

/// TCP Client representing the Viewer side.
pub struct TcpViewerClient;

impl TcpViewerClient {
    /// Connects to the host, verifies handshake, and returns channels for video & input.
    pub async fn connect(
        host_addr: SocketAddr,
        token: String,
        disable_tls: bool,
        tls_fingerprint: Option<String>,
    ) -> Result<
        (
            HandshakeResponse,
            tokio::sync::mpsc::Receiver<EncodedVideoPacket>,
            tokio::sync::mpsc::Sender<InputEvent>,
        ),
        NetError,
    > {
        let stream = TcpStream::connect(host_addr).await?;
        info!("TCP socket connected to host at {host_addr}");

        if !disable_tls {
            let tls_config = create_client_tls_config(tls_fingerprint);
            let connector = TlsConnector::from(tls_config);
            let domain = ServerName::try_from("agentdesk.local".to_string())
                .map_err(|e| NetError::TlsError(format!("Invalid DNS name: {e}")))?;

            let tls_stream = connector.connect(domain, stream).await.map_err(|e| {
                NetError::TlsError(format!("TLS handshake with {host_addr} failed: {e}"))
            })?;

            let framed = Framed::new(tls_stream, FramingCodec::new());
            perform_handshake_and_split(framed, token).await
        } else {
            warn!("Connecting with plain TCP (TLS disabled)!");
            let framed = Framed::new(stream, FramingCodec::new());
            perform_handshake_and_split(framed, token).await
        }
    }
}

async fn perform_handshake_and_split<S>(
    mut framed: Framed<S, FramingCodec>,
    token: String,
) -> Result<
    (
        HandshakeResponse,
        tokio::sync::mpsc::Receiver<EncodedVideoPacket>,
        tokio::sync::mpsc::Sender<InputEvent>,
    ),
    NetError,
>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    // Step 1: Send HandshakeRequest
    let req = HandshakeRequest {
        token,
        client_version: env!("CARGO_PKG_VERSION").to_string(),
        requested_permissions: Some(SessionPermissions::standard()),
    };
    let req_payload = serde_json::to_vec(&req)
        .map_err(|e| NetError::HandshakeFailed(e.to_string()))?;

    framed
        .send((MsgType::Handshake, req_payload))
        .await
        .map_err(NetError::Core)?;

    // Step 2: Read HandshakeResponse
    let (msg_type, payload) = framed
        .next()
        .await
        .ok_or(NetError::ConnectionClosed)?
        .map_err(NetError::Core)?;

    if msg_type != MsgType::Handshake {
        return Err(NetError::HandshakeFailed(format!(
            "Expected HandshakeResponse, got {msg_type:?}"
        )));
    }

    let resp: HandshakeResponse = serde_json::from_slice(&payload)
        .map_err(|e| NetError::HandshakeFailed(format!("Invalid response JSON: {e}")))?;

    if !resp.success {
        return Err(NetError::AuthenticationFailed(
            resp.reason.unwrap_or_else(|| "Unknown failure".to_string()),
        ));
    }

    info!("Handshake successful! Host screen: {:?}", resp.screen);

    let (mut sink, mut stream) = framed.split();
    let (video_tx, video_rx) = tokio::sync::mpsc::channel::<EncodedVideoPacket>(64);
    let (input_tx, mut input_rx) = tokio::sync::mpsc::channel::<InputEvent>(128);

    // Incoming frame receiver loop
    tokio::spawn(async move {
        while let Some(item) = stream.next().await {
            match item {
                Ok((MsgType::VideoFrame, payload)) => {
                    if let Ok(packet) = serde_json::from_slice::<EncodedVideoPacket>(&payload) {
                        if video_tx.send(packet).await.is_err() {
                            break;
                        }
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    warn!("Stream error: {e}");
                    break;
                }
            }
        }
    });

    // Outgoing input event sender loop
    tokio::spawn(async move {
        while let Some(event) = input_rx.recv().await {
            if let Ok(payload) = serde_json::to_vec(&event) {
                if sink.send((MsgType::InputEvent, payload)).await.is_err() {
                    break;
                }
            }
        }
    });

    Ok((resp, video_rx, input_tx))
}
