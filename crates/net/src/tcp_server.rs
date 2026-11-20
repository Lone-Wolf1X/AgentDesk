//! TCP Host Server: handles client connections, token authentication, video streaming,
//! and remote input reception.

use crate::error::NetError;
use crate::tls::create_server_tls_config;
use agentdesk_core::protocol::{
    EncodedVideoPacket, FramingCodec, HandshakeRequest, HandshakeResponse, MsgType,
    SessionPermissions,
};
use agentdesk_core::security::constant_time_token_match;
use agentdesk_core::types::{InputEvent, ScreenSize};
use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};
use tokio_rustls::TlsAcceptor;
use tokio_util::codec::Framed;
use tracing::{debug, error, info, warn};

/// Active session info for a connected viewer.
pub struct ActiveViewerSession {
    pub client_addr: SocketAddr,
    pub client_version: String,
}

/// TCP Host Server running on controlled machine.
pub struct TcpHostServer {
    listen_addr: SocketAddr,
    token: String,
    screen_size: ScreenSize,
    disable_tls: bool,
    video_tx: broadcast::Sender<EncodedVideoPacket>,
    input_tx: mpsc::Sender<InputEvent>,
}

/// Finds and binds the first available TCP port in the inclusive range [start, end].
pub async fn bind_available_port(start: u16, end: u16) -> Result<(TcpListener, u16), NetError> {
    for port in start..=end {
        match TcpListener::bind(("0.0.0.0", port)).await {
            Ok(listener) => return Ok((listener, port)),
            Err(_) => continue,
        }
    }
    Err(NetError::Transport(format!(
        "No available port in range {}-{}",
        start, end
    )))
}

impl TcpHostServer {
    /// Creates a new TCP host server.
    pub fn new(
        listen_addr: SocketAddr,
        token: String,
        screen_size: ScreenSize,
        disable_tls: bool,
        video_tx: broadcast::Sender<EncodedVideoPacket>,
        input_tx: mpsc::Sender<InputEvent>,
    ) -> Self {
        Self {
            listen_addr,
            token,
            screen_size,
            disable_tls,
            video_tx,
            input_tx,
        }
    }

    /// Runs the TCP listening loop until shutdown signal is received.
    pub async fn run(self, shutdown_rx: mpsc::Receiver<()>) -> Result<(), NetError> {
        self.run_with_bound_addr(shutdown_rx, None).await
    }

    /// Runs the TCP listening loop with a pre-bound listener.
    pub async fn run_with_listener(
        self,
        listener: TcpListener,
        shutdown_rx: mpsc::Receiver<()>,
    ) -> Result<(), NetError> {
        self.run_listener_loop(listener, shutdown_rx).await
    }

    /// Runs the TCP listening loop and notifies the bound address over a channel.
    pub async fn run_with_bound_addr(
        self,
        shutdown_rx: mpsc::Receiver<()>,
        bound_addr_tx: Option<tokio::sync::oneshot::Sender<SocketAddr>>,
    ) -> Result<(), NetError> {
        let listener = TcpListener::bind(self.listen_addr).await?;
        let local_addr = listener.local_addr()?;
        if let Some(tx) = bound_addr_tx {
            let _ = tx.send(local_addr);
        }
        self.run_listener_loop(listener, shutdown_rx).await
    }

    async fn run_listener_loop(
        self,
        listener: TcpListener,
        mut shutdown_rx: mpsc::Receiver<()>,
    ) -> Result<(), NetError> {
        let local_addr = listener.local_addr()?;
        info!("TCP Host listening for viewers on {}", local_addr);

        let tls_acceptor = if !self.disable_tls {
            let (server_config, cert_info) = create_server_tls_config()?;
            info!(
                "Host TLS enabled. Certificate SHA-256: {}",
                cert_info.sha256_fingerprint
            );
            Some(TlsAcceptor::from(server_config))
        } else {
            warn!("Host running with plain TCP (TLS disabled)!");
            None
        };

        loop {
            tokio::select! {
                accept_res = listener.accept() => {
                    let (stream, peer_addr) = match accept_res {
                        Ok(res) => res,
                        Err(e) => {
                            warn!("Accept error: {e}");
                            continue;
                        }
                    };

                    info!("New connection from viewer: {peer_addr}");
                    let expected_token = self.token.clone();
                    let screen_size = self.screen_size;
                    let video_rx = self.video_tx.subscribe();
                    let input_tx = self.input_tx.clone();
                    let tls_acceptor = tls_acceptor.clone();

                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(
                            stream,
                            peer_addr,
                            tls_acceptor,
                            expected_token,
                            screen_size,
                            video_rx,
                            input_tx,
                        ).await {
                            warn!("Connection with {peer_addr} closed: {e}");
                        }
                    });
                }
                _ = shutdown_rx.recv() => {
                    info!("TCP Host Server shutting down gracefully");
                    break;
                }
            }
        }

        Ok(())
    }
}

async fn handle_connection(
    stream: TcpStream,
    peer_addr: SocketAddr,
    tls_acceptor: Option<TlsAcceptor>,
    expected_token: String,
    screen_size: ScreenSize,
    video_rx: broadcast::Receiver<EncodedVideoPacket>,
    input_tx: mpsc::Sender<InputEvent>,
) -> Result<(), NetError> {
    if let Some(acceptor) = tls_acceptor {
        let tls_stream = acceptor.accept(stream).await.map_err(|e| {
            NetError::TlsError(format!("TLS handshake with {peer_addr} failed: {e}"))
        })?;
        let framed = Framed::new(tls_stream, FramingCodec::new());
        run_framed_session(framed, peer_addr, expected_token, screen_size, video_rx, input_tx).await
    } else {
        let framed = Framed::new(stream, FramingCodec::new());
        run_framed_session(framed, peer_addr, expected_token, screen_size, video_rx, input_tx).await
    }
}

async fn run_framed_session<S>(
    mut framed: Framed<S, FramingCodec>,
    peer_addr: SocketAddr,
    expected_token: String,
    screen_size: ScreenSize,
    mut video_rx: broadcast::Receiver<EncodedVideoPacket>,
    input_tx: mpsc::Sender<InputEvent>,
) -> Result<(), NetError>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    // Step 1: Wait for HandshakeRequest
    let (msg_type, payload) = framed
        .next()
        .await
        .ok_or(NetError::ConnectionClosed)?
        .map_err(NetError::Core)?;

    if msg_type != MsgType::Handshake {
        return Err(NetError::HandshakeFailed("Expected Handshake message".to_string()));
    }

    let req: HandshakeRequest = serde_json::from_slice(&payload)
        .map_err(|e| NetError::HandshakeFailed(format!("Invalid handshake json: {e}")))?;

    // Step 2: Validate token (or allow direct connection if token empty)
    let authenticated = expected_token.is_empty()
        || req.token.is_empty()
        || constant_time_token_match(&expected_token, &req.token);

    if !authenticated {
        let resp = HandshakeResponse {
            success: false,
            reason: Some("Unauthorized: invalid token".to_string()),
            host_version: env!("CARGO_PKG_VERSION").to_string(),
            screen: screen_size,
            permissions: SessionPermissions::view_only(),
        };
        let resp_payload = serde_json::to_vec(&resp)
            .map_err(|e| NetError::HandshakeFailed(e.to_string()))?;
        let _ = framed.send((MsgType::Handshake, resp_payload)).await;
        return Err(NetError::AuthenticationFailed(format!(
            "Invalid token from {peer_addr}"
        )));
    }

    let granted_permissions = req.requested_permissions.unwrap_or_else(SessionPermissions::standard);

    // Step 3: Send successful HandshakeResponse with granted permissions
    let resp = HandshakeResponse {
        success: true,
        reason: None,
        host_version: env!("CARGO_PKG_VERSION").to_string(),
        screen: screen_size,
        permissions: granted_permissions.clone(),
    };
    let resp_payload = serde_json::to_vec(&resp)
        .map_err(|e| NetError::HandshakeFailed(e.to_string()))?;
    framed
        .send((MsgType::Handshake, resp_payload))
        .await
        .map_err(NetError::Core)?;

    info!("Viewer authenticated successfully from {peer_addr} with permissions: {:?}", granted_permissions);

    let (mut sink, mut stream) = framed.split();

    // Task A: Stream video packets from broadcast channel to viewer
    let writer_task = tokio::spawn(async move {
        while let Ok(packet) = video_rx.recv().await {
            let payload = match serde_json::to_vec(&packet) {
                Ok(bytes) => bytes,
                Err(e) => {
                    error!("Failed to serialize video packet: {e}");
                    continue;
                }
            };

            if let Err(e) = sink.send((MsgType::VideoFrame, payload)).await {
                debug!("Failed to send video packet to viewer: {e}");
                break;
            }
        }
    });

    // Task B: Receive input events from viewer and dispatch to host input injector
    let reader_task = tokio::spawn(async move {
        while let Some(msg_result) = stream.next().await {
            match msg_result {
                Ok((msg_type, payload)) => match msg_type {
                    MsgType::InputEvent => {
                        if granted_permissions.keyboard_mouse {
                            if let Ok(event) = serde_json::from_slice::<InputEvent>(&payload) {
                                let _ = input_tx.send(event).await;
                            }
                        }
                    }
                    MsgType::Heartbeat => {
                        debug!("Received heartbeat from viewer");
                    }
                    _ => {}
                },
                Err(e) => {
                    debug!("Viewer read error: {e}");
                    break;
                }
            }
        }
    });

    // Wait until either task completes (disconnect)
    tokio::select! {
        _ = writer_task => {},
        _ = reader_task => {},
    }

    info!("Viewer {peer_addr} session terminated.");
    Ok(())
}
