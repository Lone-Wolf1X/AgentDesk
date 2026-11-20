// AgentDesk — Free & Open Source Remote Desktop
// Copyright (c) 2025 Next Gen Innovations Private Limited
// Developer: Abhishek Kumar Paswan
// Website: https://nextgeninnovations.com.np/
// License: MIT — see LICENSE file
// Made in Nepal 🇳🇵

use agentdesk_capture::{check_capture_permissions, MockCapturer, ScapCapturer, ScreenCapturer};
use agentdesk_core::config::CaptureConfig;
use agentdesk_core::protocol::EncodedVideoPacket;
use agentdesk_core::types::{InputEvent, ScreenSize};
use agentdesk_encode::{OpenH264Encoder, VideoEncoder};
use agentdesk_gui::HostGuiState;
use agentdesk_input::{check_input_permissions, InputListener, InputInjector, MockInjector, RdevInjector, RdevListener};
use agentdesk_net::TcpHostServer;
use anyhow::{Context, Result};
use clap::Parser;
use rand::Rng;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(
    name = "agentdesk-host",
    version = "0.1.0",
    about = "AgentDesk Host Server — Free & Open Source Remote Desktop — Made in Nepal 🇳🇵\n(c) 2025 Next Gen Innovations Private Limited — https://nextgeninnovations.com.np/\nDeveloper: Abhishek Kumar Paswan"
)]
struct Cli {
    /// TCP listen address for viewers
    #[arg(short, long, default_value = "0.0.0.0:9090")]
    listen: SocketAddr,

    /// Access password / token for remote control
    #[arg(short, long, env = "AGENTDESK_TOKEN", default_value = "")]
    token: String,

    /// Target capture frames per second
    #[arg(long, default_value_t = 20)]
    fps: u32,

    /// Video bitrate in kbps
    #[arg(long, default_value_t = 2000)]
    bitrate: u32,

    /// Display index to capture
    #[arg(long, default_value_t = 0)]
    display: usize,

    /// Run in headless mode without GUI (ideal for remote servers)
    #[arg(long, default_value_t = false)]
    headless: bool,

    /// Optional 9-digit ID override
    #[arg(long)]
    id: Option<String>,

    /// Disable TLS encryption (plain TCP, for local LAN testing)
    #[arg(long, default_value_t = false)]
    no_tls: bool,

    /// Use synthetic mock display (useful for CI and headless VMs)
    #[arg(long, default_value_t = false)]
    mock_display: bool,
}

fn print_branded_banner() {
    println!(
        "\n=======================================================================\n\
         🇳🇵 AgentDesk Host v0.1.0\n\
         Free & Open Source Remote Desktop — Made in Nepal 🇳🇵\n\
         Copyright (c) 2025 Next Gen Innovations Private Limited\n\
         Website: https://nextgeninnovations.com.np/\n\
         Developer & Maintainer: Abhishek Kumar Paswan\n\
         License: MIT — 100% Free, No Limits, No Telemetry, No Account Required\n\
         =======================================================================\n"
    );
}

fn generate_9_digit_id() -> String {
    let mut rng = rand::thread_rng();
    let num: u32 = rng.gen_range(100_000_000..=999_999_999);
    format!("{num}")
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    print_branded_banner();

    let cli = Cli::parse();

    // Verify platform permissions
    if !cli.mock_display {
        if let Err(e) = check_capture_permissions() {
            eprintln!("\n❌ Capture permission error: {e}");
            std::process::exit(1);
        }
        if let Err(e) = check_input_permissions() {
            eprintln!("\n❌ Input simulation permission error: {e}");
            std::process::exit(1);
        }
    }

    let device_id = cli.id.unwrap_or_else(generate_9_digit_id);
    let initial_password = if cli.token.is_empty() {
        "pass123".to_string()
    } else {
        cli.token
    };

    info!("Assigned Connection ID: {device_id}");

    let rt = tokio::runtime::Runtime::new().context("Failed to initialize Tokio runtime")?;

    // Background server setup
    let listen_addr = cli.listen;
    let fps = cli.fps;
    let bitrate = cli.bitrate;
    let mock = cli.mock_display;
    let no_tls = cli.no_tls;
    let pass_clone = initial_password.clone();

    let (video_tx, _) = broadcast::channel::<EncodedVideoPacket>(64);
    let (input_tx, mut input_rx) = mpsc::channel::<InputEvent>(128);

    let v_tx = video_tx.clone();

    // Spawn async background tasks
    rt.spawn(async move {
        let screen_size = ScreenSize { width: 1920, height: 1080 };
        let mut capturer: Box<dyn ScreenCapturer> = if mock {
            Box::new(MockCapturer::new(1920, 1080, CaptureConfig::default()))
        } else {
            match ScapCapturer::new(CaptureConfig::default()) {
                Ok(c) => Box::new(c),
                Err(_) => Box::new(MockCapturer::new(1920, 1080, CaptureConfig::default())),
            }
        };

        let mut encoder: Box<dyn VideoEncoder> = match OpenH264Encoder::new(1920, 1080, bitrate, fps as f32) {
            Ok(e) => Box::new(e),
            Err(_) => Box::new(agentdesk_encode::MockEncoder::new(1920, 1080)),
        };

        // Screen capture loop
        let broadcast_sender = v_tx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_micros((1_000_000.0 / fps as f64) as u64));
            loop {
                interval.tick().await;
                if let Ok(frame) = capturer.capture_frame() {
                    if let Ok(packet) = encoder.encode_frame(&frame) {
                        let _ = broadcast_sender.send(packet);
                    }
                }
            }
        });

        // Input simulation consumer
        let mut injector: Box<dyn InputInjector> = if mock {
            Box::new(MockInjector::new())
        } else {
            match RdevInjector::new() {
                Ok(inj) => Box::new(inj),
                Err(_) => Box::new(MockInjector::new()),
            }
        };

        tokio::spawn(async move {
            while let Some(event) = input_rx.recv().await {
                match event {
                    InputEvent::MouseMove { x, y } => { let _ = injector.move_mouse(x, y); }
                    InputEvent::MouseDown { button, .. } => { let _ = injector.click(button, true); }
                    InputEvent::MouseUp { button, .. } => { let _ = injector.click(button, false); }
                    InputEvent::MouseWheel { delta_x, delta_y } => { let _ = injector.mouse_scroll(delta_x, delta_y); }
                    InputEvent::KeyDown { key } => { let _ = injector.key_press(key, true); }
                    InputEvent::KeyUp { key } => { let _ = injector.key_press(key, false); }
                    InputEvent::TextInput { text } => { let _ = injector.type_text(&text); }
                }
            }
        });

        let tcp_server = TcpHostServer::new(
            listen_addr,
            pass_clone,
            screen_size,
            no_tls,
            v_tx,
            input_tx,
        );

        let (_shutdown_tx, shutdown_rx) = mpsc::channel::<()>(1);
        let _ = tcp_server.run(shutdown_rx).await;
    });

    if cli.headless {
        info!("Running in headless mode. Waiting for connections on {}...", cli.listen);
        rt.block_on(async {
            tokio::signal::ctrl_c().await.unwrap();
            info!("Shutting down host...");
        });
    } else {
        // Run egui desktop window
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([460.0, 360.0])
                .with_title("AgentDesk Host — Next Gen Innovations"),
            ..Default::default()
        };

        let mut app_state = HostGuiState::new(device_id, initial_password);

        eframe::run_simple_native("AgentDesk Host", options, move |ctx, _frame| {
            app_state.render(ctx);
        })
        .map_err(|e| anyhow::anyhow!("GUI execution failed: {e}"))?;
    }

    Ok(())
}
