//! Network transport layer for AgentDesk: TCP streaming, TLS encryption,
//! and WebRTC abstraction.

pub mod error;
pub mod tcp_client;
pub mod tcp_server;
pub mod tls;
pub mod traits;
pub mod webrtc_stub;

pub use error::NetError;
pub use tcp_client::TcpViewerClient;
pub use tcp_server::{ActiveViewerSession, TcpHostServer};
pub use tls::{create_client_tls_config, create_server_tls_config};
pub use traits::Transport;
pub use webrtc_stub::WebRtcTransportStub;
