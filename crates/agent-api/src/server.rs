//! Axum HTTP Server initialization and routing configuration.

use crate::auth::require_auth;
use crate::error::ApiError;
use crate::routes::{
    click_handler, health_handler, key_handler, move_handler, screen_size_handler,
    screenshot_handler, type_handler,
};
use agentdesk_capture::ScreenCapturer;
use agentdesk_input::InputInjector;
use axum::middleware::from_fn;
use axum::routing::{get, post};
use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

/// Shared state available across all Axum request handlers.
#[derive(Clone)]
pub struct AppState {
    pub capturer: Arc<Mutex<Box<dyn ScreenCapturer>>>,
    pub injector: Arc<Mutex<Box<dyn InputInjector>>>,
}

/// Starts the local Agent API HTTP server.
pub async fn start_agent_api(
    bind_addr: SocketAddr,
    token: String,
    capturer: Box<dyn ScreenCapturer>,
    injector: Box<dyn InputInjector>,
    mut shutdown_rx: mpsc::Receiver<()>,
) -> Result<(), ApiError> {
    // Security check: warn if binding to 0.0.0.0
    if bind_addr.ip().is_unspecified() {
        warn!(
            "*****************************************************************\n\
             * SECURITY WARNING: Agent API is bound to 0.0.0.0!               *\n\
             * The Agent API exposes full desktop control over the network.  *\n\
             * It is strongly recommended to bind strictly to 127.0.0.1.     *\n\
             *****************************************************************"
        );
    } else {
        info!("Agent API binding securely to local loopback: {bind_addr}");
    }

    let shared_state = AppState {
        capturer: Arc::new(Mutex::new(capturer)),
        injector: Arc::new(Mutex::new(injector)),
    };

    let expected_token = token.clone();

    // Authenticated routes
    let protected_routes = Router::new()
        .route("/screenshot", post(screenshot_handler))
        .route("/click", post(click_handler))
        .route("/move", post(move_handler))
        .route("/type", post(type_handler))
        .route("/key", post(key_handler))
        .route("/screen-size", get(screen_size_handler))
        .layer(from_fn(move |req, next| {
            let tok = expected_token.clone();
            require_auth(tok, req, next)
        }));

    // Public routes (health check)
    let app = Router::new()
        .route("/health", get(health_handler))
        .merge(protected_routes)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to bind to {bind_addr}: {e}")))?;

    info!("Agent API running on http://{bind_addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.recv().await;
            info!("Agent API received shutdown signal.");
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Agent API server error: {e}")))?;

    Ok(())
}
