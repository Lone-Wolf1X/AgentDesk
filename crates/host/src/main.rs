//! AgentDesk Host Binary (`agentdesk-host`)
//!
//! Captures the screen, encodes into H.264, streams via TCP/TLS to viewers,
//! injects remote inputs, and hosts the local HTTP Agent API.

use agentdesk_capture::{
    check_capture_permissions, MockCapturer, ScapCapturer, ScreenCapturer,
};
use agentdesk_core::config::CaptureConfig;
use agentdesk_core::protocol::EncodedVideoPacket;
use agentdesk_core::types::{InputEvent, ScreenSize};
use agentdesk_encode::{OpenH264Encoder, VideoEncoder};
use agentdesk_input::{
    check_input_permissions, InputListener, InputInjector, MockInjector, MockListener,
    RdevInjector, RdevListener,
};
use agentdesk_net::TcpHostServer;
use anyhow::{Context, Result};
use clap::Parser;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(name = "agentdesk-host", author, version, about = "AgentDesk Host Server")]
struct Cli {
    /// TCP listen address for viewers
    #[arg(short, long, default_value = "0.0.0.0:9090")]
    listen: SocketAddr,

    /// Authentication token required for viewers and Agent API
    #[arg(short, long, env = "AGENTDESK_TOKEN", default_value = "agentdesk-secret-token")]
    token: String,

    /// Target capture and encoding frames per second
    #[arg(long, default_value_t = 20)]
    fps: u32,

    /// Target video bitrate in kbps
    #[arg(long, default_value_t = 2000)]
    bitrate: u32,

    /// Display index to capture
    #[arg(long, default_value_t = 0)]
    display: usize,

    /// Enable local AI Agent HTTP REST API
    #[arg(long, default_value_t = true)]
    enable_agent_api: bool,

    /// Local HTTP address for AI Agent REST API
    #[arg(long, default_value = "127.0.0.1:7878")]
    agent_api_addr: SocketAddr,

    /// Disable TLS encryption (use plain TCP, useful for local testing)
    #[arg(long, default_value_t = false)]
    no_tls: bool,

    /// Use synthetic mock display instead of physical display (useful for CI/headless)
    #[arg(long, default_value_t = false)]
    mock_display: bool,

    /// Enable session lock: pause remote input if local user moves mouse or types
    #[arg(long, default_value_t = false)]
    session_lock: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,agentdesk_host=debug,agentdesk_net=debug".into()),
        )
        .init();

    let cli = Cli::parse();
    info!("Starting AgentDesk Host v{}", env!("CARGO_PKG_VERSION"));
    info!("Configured FPS: {}, Bitrate: {} kbps", cli.fps, cli.bitrate);

    // Step 1: Preflight checks (permissions, Wayland detection, accessibility)
    if !cli.mock_display {
        if let Err(e) = check_capture_permissions() {
            eprintln!("\n❌ CAPTURE INITIALIZATION FAILED:");
            eprintln!("{e}\n");
            std::process::exit(1);
        }

        if let Err(e) = check_input_permissions() {
            eprintln!("\n❌ INPUT INJECTION PERMISSION FAILED:");
            eprintln!("{e}\n");
            std::process::exit(1);
        }
    } else {
        warn!("Running in --mock-display mode: synthetic display & mock input enabled");
    }

    // Step 2: Initialize capturer and injector
    let capture_cfg = CaptureConfig {
        fps: cli.fps,
        display_index: cli.display,
        crop_region: None,
    };

    let mut capturer: Box<dyn ScreenCapturer> = if cli.mock_display {
        Box::new(MockCapturer::new(1920, 1080, capture_cfg))
    } else {
        match ScapCapturer::new(capture_cfg.clone()) {
            Ok(c) => Box::new(c),
            Err(e) => {
                warn!("Physical scap capturer failed ({e}), falling back to mock capturer");
                Box::new(MockCapturer::new(1920, 1080, capture_cfg))
            }
        }
    };

    let display_info = capturer
        .display_info()
        .context("Failed to query display information")?;
    let screen_size = ScreenSize {
        width: display_info.width,
        height: display_info.height,
    };
    info!(
        "Capturing display '{}' ({}x{})",
        display_info.name, screen_size.width, screen_size.height
    );

    // Step 3: Initialize encoder
    let mut encoder: Box<dyn VideoEncoder> = match OpenH264Encoder::new(
        screen_size.width,
        screen_size.height,
        cli.bitrate,
        cli.fps as f32,
    ) {
        Ok(enc) => Box::new(enc),
        Err(e) => {
            warn!("Failed to initialize OpenH264 ({e}), falling back to mock encoder");
            Box::new(agentdesk_encode::MockEncoder::new(
                screen_size.width,
                screen_size.height,
            ))
        }
    };

    // Channels
    let (video_broadcast_tx, _) = broadcast::channel::<EncodedVideoPacket>(64);
    let (input_event_tx, mut input_event_rx) = mpsc::channel::<InputEvent>(128);
    let (shutdown_tx, _) = broadcast::channel::<()>(1);

    // Step 4: Session locking mechanism
    let remote_input_paused = Arc::new(AtomicBool::new(false));
    if cli.session_lock {
        let paused_flag = remote_input_paused.clone();
        if cli.mock_display {
            let listener = MockListener::new();
            let _ = listener.start_listening(move |_| {});
        } else {
            let listener = RdevListener::new();
            let flag = paused_flag.clone();
            if let Err(e) = listener.start_listening(move |_event| {
                flag.store(true, Ordering::Relaxed);
            }) {
                warn!("Failed to start local input listener for session lock: {e}");
            }
        }
    }

    // Step 5: Input injection consumer task
    let mut injector: Box<dyn InputInjector> = if cli.mock_display {
        Box::new(MockInjector::new())
    } else {
        match RdevInjector::new() {
            Ok(inj) => Box::new(inj),
            Err(_) => Box::new(MockInjector::new()),
        }
    };

    let remote_paused = remote_input_paused.clone();
    tokio::spawn(async move {
        while let Some(event) = input_event_rx.recv().await {
            if remote_paused.load(Ordering::Relaxed) {
                warn!("Remote input dropped: session locked due to local user activity");
                continue;
            }

            match event {
                InputEvent::MouseMove { x, y } => {
                    let _ = injector.move_mouse(x, y);
                }
                InputEvent::MouseDown { button, .. } => {
                    let _ = injector.click(button, true);
                }
                InputEvent::MouseUp { button, .. } => {
                    let _ = injector.click(button, false);
                }
                InputEvent::MouseWheel { delta_x, delta_y } => {
                    let _ = injector.mouse_scroll(delta_x, delta_y);
                }
                InputEvent::KeyDown { key } => {
                    let _ = injector.key_press(key, true);
                }
                InputEvent::KeyUp { key } => {
                    let _ = injector.key_press(key, false);
                }
                InputEvent::TextInput { text } => {
                    let _ = injector.type_text(&text);
                }
            }
        }
    });

    // Step 6: Screen capture + encode loop
    let capture_broadcast_tx = video_broadcast_tx.clone();
    let target_interval = Duration::from_micros((1_000_000.0 / cli.fps as f64) as u64);
    let mut shutdown_rx_capture = shutdown_tx.subscribe();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(target_interval);
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    match capturer.capture_frame() {
                        Ok(frame) => {
                            match encoder.encode_frame(&frame) {
                                Ok(packet) => {
                                    // Send to broadcast channel if there are active viewers
                                    let _ = capture_broadcast_tx.send(packet);
                                }
                                Err(e) => {
                                    error!("Frame encode error: {e}");
                                }
                            }
                        }
                        Err(e) => {
                            error!("Frame capture error: {e}");
                        }
                    }
                }
                _ = shutdown_rx_capture.recv() => {
                    info!("Capture & Encode loop shutting down");
                    break;
                }
            }
        }
    });

    // Step 7: Start TCP Host Server
    let tcp_server = TcpHostServer::new(
        cli.listen,
        cli.token.clone(),
        screen_size,
        cli.no_tls,
        video_broadcast_tx,
        input_event_tx,
    );

    let (tcp_shutdown_tx, tcp_shutdown_rx) = mpsc::channel::<()>(1);
    let tcp_task = tokio::spawn(async move {
        if let Err(e) = tcp_server.run(tcp_shutdown_rx).await {
            error!("TCP Host server exited with error: {e}");
        }
    });

    // Step 8: Start Agent REST API if enabled
    let (agent_shutdown_tx, agent_shutdown_rx) = mpsc::channel::<()>(1);
    let agent_task = if cli.enable_agent_api {
        let api_capturer: Box<dyn ScreenCapturer> = if cli.mock_display {
            Box::new(MockCapturer::new(1920, 1080, CaptureConfig::default()))
        } else {
            match ScapCapturer::new(CaptureConfig::default()) {
                Ok(c) => Box::new(c),
                Err(_) => Box::new(MockCapturer::new(1920, 1080, CaptureConfig::default())),
            }
        };

        let api_injector: Box<dyn InputInjector> = if cli.mock_display {
            Box::new(MockInjector::new())
        } else {
            match RdevInjector::new() {
                Ok(i) => Box::new(i),
                Err(_) => Box::new(MockInjector::new()),
            }
        };

        let token = cli.token.clone();
        let api_addr = cli.agent_api_addr;

        Some(tokio::spawn(async move {
            if let Err(e) = agentdesk_agent_api::start_agent_api(
                api_addr,
                token,
                api_capturer,
                api_injector,
                agent_shutdown_rx,
            )
            .await
            {
                error!("Agent API error: {e}");
            }
        }))
    } else {
        None
    };

    info!("AgentDesk Host is fully operational. Press Ctrl+C to stop.");

    // Step 9: Wait for Ctrl+C for graceful shutdown
    tokio::signal::ctrl_c()
        .await
        .context("Failed to listen for Ctrl+C")?;

    info!("Shutdown signal received. Terminating all subsystems...");
    let _ = shutdown_tx.send(());
    let _ = tcp_shutdown_tx.send(()).await;
    let _ = agent_shutdown_tx.send(()).await;

    let _ = tcp_task.await;
    if let Some(t) = agent_task {
        let _ = t.await;
    }

    info!("AgentDesk Host shutdown complete.");
    Ok(())
}
