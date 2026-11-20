// AgentDesk — Free & Open Source Remote Desktop
// Copyright (c) 2025 Next Gen Innovations Private Limited
// Developer: Abhishek Kumar Paswan
// Website: https://nextgeninnovations.com.np/
// License: MIT — see LICENSE file
// Made in Nepal 🇳🇵

use agentdesk_capture::{MockCapturer, ScapCapturer, ScreenCapturer};
use agentdesk_core::config::CaptureConfig;
use agentdesk_core::network_info::get_local_ip;
use agentdesk_core::protocol::EncodedVideoPacket;
use agentdesk_core::types::{InputEvent, ScreenSize};
use agentdesk_encode::{MockDecoder, MockEncoder, OpenH264Decoder, OpenH264Encoder, VideoDecoder, VideoEncoder};
use agentdesk_gui::{AgentDeskApp, AppEvent, UiAction};
use agentdesk_input::{InputInjector, MockInjector, RdevInjector};
use agentdesk_net::{bind_available_port, TcpHostServer, TcpViewerClient};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rand::Rng;
use std::net::SocketAddr;
use std::sync::mpsc::channel;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(
    name = "agentdesk",
    version = "0.1.0",
    about = "Free & Open Source Remote Desktop — Made in Nepal 🇳🇵\nCopyright (c) 2025 Next Gen Innovations Private Limited\nDeveloper: Abhishek Kumar Paswan"
)]
struct Cli {
    #[command(subcommand)]
    mode: Option<Mode>,
}

#[derive(Subcommand, Debug)]
enum Mode {
    /// Start host role (screen sharing & remote control server)
    Host {
        #[arg(long, default_value = "0.0.0.0:9090")]
        listen: String,
        #[arg(long, env = "AGENTDESK_TOKEN")]
        token: Option<String>,
        #[arg(long, default_value_t = 20)]
        fps: u32,
        #[arg(long, default_value_t = 2000)]
        bitrate: u32,
        #[arg(long)]
        headless: bool,
        #[arg(long)]
        no_tls: bool,
    },
    /// Connect to remote host
    Connect {
        #[arg(long)]
        connect: Option<String>,
        #[arg(long)]
        id: Option<String>,
        #[arg(long)]
        no_tls: bool,
    },
    /// Start app with host listening enabled (same as default)
    Serve {
        #[arg(long, env = "AGENTDESK_TOKEN")]
        token: Option<String>,
    },
}

fn print_branded_banner() {
    println!(
        "\n=======================================================================\n\
         🇳🇵 AgentDesk v0.1.0 — Unified Remote Desktop\n\
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

    match cli.mode {
        Some(Mode::Host {
            listen,
            token,
            fps,
            bitrate,
            headless,
            no_tls,
        }) if headless => {
            let rt = tokio::runtime::Runtime::new().context("Failed to start Tokio runtime")?;
            rt.block_on(run_headless_host(listen, token, fps, bitrate, no_tls))
        }
        Some(Mode::Connect {
            connect,
            id,
            no_tls,
        }) => {
            run_gui(connect, id, no_tls)
        }
        _ => {
            run_gui(None, None, false)
        }
    }
}

async fn run_headless_host(
    listen: String,
    token: Option<String>,
    fps: u32,
    bitrate: u32,
    no_tls: bool,
) -> Result<()> {
    let password = token.unwrap_or_default();
    let listen_addr: SocketAddr = listen.parse().context("Invalid listen address")?;

    info!("Starting AgentDesk Headless Host...");
    info!("Listen Address: {listen_addr}");

    let (video_tx, _) = broadcast::channel::<EncodedVideoPacket>(64);
    let (input_tx, mut input_rx) = mpsc::channel::<InputEvent>(128);

    let v_tx = video_tx.clone();
    tokio::spawn(async move {
        let mut capturer: Box<dyn ScreenCapturer> = match ScapCapturer::new(CaptureConfig::default()) {
            Ok(c) => Box::new(c),
            Err(_) => Box::new(MockCapturer::new(1920, 1080, CaptureConfig::default())),
        };
        let mut encoder: Box<dyn VideoEncoder> = match OpenH264Encoder::new(1920, 1080, bitrate, fps as f32) {
            Ok(e) => Box::new(e),
            Err(_) => Box::new(MockEncoder::new(1920, 1080)),
        };

        loop {
            if let Ok(frame) = capturer.capture_frame() {
                if let Ok(packet) = encoder.encode_frame(&frame) {
                    let _ = v_tx.send(packet);
                }
            }
            tokio::time::sleep(Duration::from_millis(1000 / fps as u64)).await;
        }
    });

    tokio::spawn(async move {
        let mut injector: Box<dyn InputInjector> = match RdevInjector::new() {
            Ok(inj) => Box::new(inj),
            Err(_) => Box::new(MockInjector::new()),
        };
        while let Some(event) = input_rx.recv().await {
            let _ = injector.inject_event(event);
        }
    });

    let screen_size = ScreenSize { width: 1920, height: 1080 };
    let server = TcpHostServer::new(listen_addr, password, screen_size, no_tls, video_tx, input_tx);
    let (_shutdown_tx, shutdown_rx) = mpsc::channel::<()>(1);
    server.run(shutdown_rx).await.context("Server error")?;

    Ok(())
}

fn run_gui(
    auto_connect: Option<String>,
    auto_id: Option<String>,
    no_tls: bool,
) -> Result<()> {
    let rt = tokio::runtime::Runtime::new().context("Failed to start Tokio runtime")?;

    let local_ip = get_local_ip();
    let my_id = generate_9_digit_id();

    // 1. Auto-bind port in range 9090-9100
    let (listener, local_port) = rt.block_on(async {
        bind_available_port(9090, 9100).await.unwrap_or_else(|_| {
            let std_listener = std::net::TcpListener::bind("0.0.0.0:0").unwrap();
            std_listener.set_nonblocking(true).unwrap();
            let port = std_listener.local_addr().unwrap().port();
            let tokio_listener = tokio::net::TcpListener::from_std(std_listener).unwrap();
            (tokio_listener, port)
        })
    });

    let (ui_action_tx, ui_action_rx) = channel::<UiAction>();
    let (app_event_tx, app_event_rx) = channel::<AppEvent>();
    let (frame_tx, frame_rx) = channel::<(u32, u32, Vec<u8>)>();

    // 2. Start background Host Server task
    let (video_broadcast_tx, _) = broadcast::channel::<EncodedVideoPacket>(64);
    let (host_input_tx, mut host_input_rx) = mpsc::channel::<InputEvent>(128);

    let v_tx = video_broadcast_tx.clone();
    rt.spawn(async move {
        let mut capturer: Box<dyn ScreenCapturer> = match ScapCapturer::new(CaptureConfig::default()) {
            Ok(c) => Box::new(c),
            Err(_) => Box::new(MockCapturer::new(1920, 1080, CaptureConfig::default())),
        };
        let mut encoder: Box<dyn VideoEncoder> = match OpenH264Encoder::new(1920, 1080, 2000, 20.0) {
            Ok(e) => Box::new(e),
            Err(_) => Box::new(MockEncoder::new(1920, 1080)),
        };

        loop {
            if let Ok(frame) = capturer.capture_frame() {
                if let Ok(packet) = encoder.encode_frame(&frame) {
                    let _ = v_tx.send(packet);
                }
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });

    rt.spawn(async move {
        let mut injector: Box<dyn InputInjector> = match RdevInjector::new() {
            Ok(inj) => Box::new(inj),
            Err(_) => Box::new(MockInjector::new()),
        };
        while let Some(event) = host_input_rx.recv().await {
            let _ = injector.inject_event(event);
        }
    });

    rt.spawn(async move {
        let screen_size = ScreenSize { width: 1920, height: 1080 };
        let server = TcpHostServer::new(
            "0.0.0.0:0".parse().unwrap(),
            String::new(), // Direct connection, prompt-based accept
            screen_size,
            no_tls,
            video_broadcast_tx,
            host_input_tx,
        );
        let (_shutdown_tx, shutdown_rx) = mpsc::channel::<()>(1);
        let _ = server.run_with_listener(listener, shutdown_rx).await;
    });

    // 3. UI Action processing loop
    let f_tx = frame_tx.clone();
    let evt_tx = app_event_tx.clone();
    let rt_handle = rt.handle().clone();
    std::thread::spawn(move || {
        while let Ok(action) = ui_action_rx.recv() {
            match action {
                UiAction::Connect { target } => {
                    info!("Initiating connection to {target}...");
                    let ft = f_tx.clone();
                    let et = evt_tx.clone();
                    let target_clone = target.clone();
                    rt_handle.spawn(async move {
                        let cleaned = target_clone.replace([' ', '-'], "");
                        let parsed_addr = if let Ok(addr) = target_clone.parse::<SocketAddr>() {
                            Some(addr)
                        } else if let Ok(ip) = target_clone.parse::<std::net::IpAddr>() {
                            Some(SocketAddr::new(ip, 9090))
                        } else if let Ok(addr) = cleaned.parse::<SocketAddr>() {
                            Some(addr)
                        } else if let Ok(ip) = cleaned.parse::<std::net::IpAddr>() {
                            Some(SocketAddr::new(ip, 9090))
                        } else {
                            None
                        };

                        if let Some(addr) = parsed_addr {
                            match TcpViewerClient::connect(addr, String::new(), no_tls, None).await {
                                Ok((handshake, mut video_rx, net_in_tx)) => {
                                    info!("Connected to remote host {addr}! Screen: {:?}", handshake.screen);
                                    let _ = et.send(AppEvent::Connected {
                                        remote_addr: addr.to_string(),
                                        permissions: handshake.permissions,
                                        screen_size: (handshake.screen.width as f64, handshake.screen.height as f64),
                                        input_tx: net_in_tx,
                                    });

                                    let mut decoder: Box<dyn VideoDecoder> = match OpenH264Decoder::new() {
                                        Ok(d) => Box::new(d),
                                        Err(_) => Box::new(MockDecoder::new()),
                                    };
                                    tokio::spawn(async move {
                                        while let Some(packet) = video_rx.recv().await {
                                            if let Ok(frame) = decoder.decode_packet(&packet) {
                                                let _ = ft.send((frame.width, frame.height, frame.to_rgba()));
                                            }
                                        }
                                    });
                                }
                                Err(e) => {
                                    error!("Connection to {addr} failed: {e}");
                                    let _ = et.send(AppEvent::ConnectionFailed {
                                        error: e.to_string(),
                                    });
                                }
                            }
                        } else {
                            warn!("Could not parse target '{target_clone}' as IP or IP:PORT");
                            let _ = et.send(AppEvent::ConnectionFailed {
                                error: format!("Invalid address: '{target_clone}'"),
                            });
                        }
                    });
                }
                UiAction::Disconnect => {
                    info!("Disconnect requested");
                    let _ = evt_tx.send(AppEvent::Disconnected);
                }
                _ => {}
            }
        }
    });

    // 4. Create eframe App
    let mut app = AgentDeskApp::new(
        local_ip,
        local_port,
        Some(my_id),
        Some(ui_action_tx.clone()),
        Some(app_event_rx),
        Some(frame_rx),
    );

    if let Some(target) = auto_connect.or(auto_id) {
        app.remote_target_input = target.clone();
        let _ = ui_action_tx.send(UiAction::Connect { target });
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([920.0, 580.0])
            .with_icon(agentdesk_gui::get_app_icon())
            .with_title("AgentDesk — Made in Nepal 🇳🇵 (Next Gen Innovations)"),
        ..Default::default()
    };

    eframe::run_native(
        "AgentDesk",
        native_options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
    .map_err(|e| anyhow::anyhow!("Eframe error: {e}"))
}
