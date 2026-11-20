// AgentDesk — Free & Open Source Remote Desktop
// Copyright (c) 2025 Next Gen Innovations Private Limited
// Developer: Abhishek Kumar Paswan
// Website: https://nextgeninnovations.com.np/
// License: MIT — see LICENSE file
// Made in Nepal 🇳🇵

use agentdesk_core::types::InputEvent;
use agentdesk_encode::{OpenH264Decoder, VideoDecoder};
use agentdesk_gui::ViewerGuiState;
use agentdesk_net::TcpViewerClient;
use anyhow::{Context, Result};
use clap::Parser;
use std::net::SocketAddr;
use std::sync::mpsc::channel;
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(
    name = "agentdesk-viewer",
    version = "0.1.0",
    about = "AgentDesk Viewer — Free & Open Source Remote Desktop — Made in Nepal 🇳🇵\n(c) 2025 Next Gen Innovations Private Limited — https://nextgeninnovations.com.np/\nDeveloper: Abhishek Kumar Paswan"
)]
struct Cli {
    /// Remote host address to connect to directly (e.g. 127.0.0.1:9090)
    #[arg(short, long, default_value = "127.0.0.1:9090")]
    connect: SocketAddr,

    /// 9-digit Connection ID (to resolve via signaling server)
    #[arg(long)]
    id: Option<String>,

    /// Access password for host
    #[arg(short, long, env = "AGENTDESK_TOKEN", default_value = "pass123")]
    password: String,

    /// Disable TLS encryption (connect via plain TCP)
    #[arg(long, default_value_t = false)]
    no_tls: bool,

    /// Start in fullscreen mode
    #[arg(long, default_value_t = false)]
    fullscreen: bool,
}

fn print_branded_banner() {
    println!(
        "\n=======================================================================\n\
         🇳🇵 AgentDesk Viewer v0.1.0\n\
         Free & Open Source Remote Desktop — Made in Nepal 🇳🇵\n\
         Copyright (c) 2025 Next Gen Innovations Private Limited\n\
         Website: https://nextgeninnovations.com.np/\n\
         Developer & Maintainer: Abhishek Kumar Paswan\n\
         License: MIT — 100% Free, No Limits, No Telemetry, No Account Required\n\
         =======================================================================\n"
    );
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    print_branded_banner();

    let cli = Cli::parse();
    let rt = tokio::runtime::Runtime::new().context("Failed to start Tokio runtime")?;

    let (frame_tx, frame_rx) = channel::<(u32, u32, Vec<u8>)>();
    let (input_tx, input_rx) = channel::<InputEvent>();

    let connect_addr = cli.connect;
    let password = cli.password.clone();
    let no_tls = cli.no_tls;

    // Spawn network connector loop
    rt.spawn(async move {
        info!("Attempting connection to remote desktop at {connect_addr}...");
        match TcpViewerClient::connect(connect_addr, password, no_tls, None).await {
            Ok((handshake, mut video_rx, net_input_tx)) => {
                info!("Connected successfully! Remote screen: {:?}", handshake.screen);

                let mut decoder: Box<dyn VideoDecoder> = match OpenH264Decoder::new() {
                    Ok(dec) => Box::new(dec),
                    Err(_) => Box::new(agentdesk_encode::MockDecoder::new()),
                };

                // Decode incoming video stream and push to GUI
                let f_tx = frame_tx.clone();
                tokio::spawn(async move {
                    while let Some(packet) = video_rx.recv().await {
                        if let Ok(frame) = decoder.decode_packet(&packet) {
                            let rgba = frame.to_rgba();
                            let _ = f_tx.send((frame.width, frame.height, rgba));
                        }
                    }
                });

                // Relay GUI inputs to host over TCP
                let mut in_rx = input_rx;
                tokio::spawn(async move {
                    while let Ok(event) = in_rx.recv() {
                        let _ = net_input_tx.send(event).await;
                    }
                });
            }
            Err(e) => {
                error!("Connection failed: {e}");
            }
        }
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_fullscreen(cli.fullscreen)
            .with_title("AgentDesk Viewer — Next Gen Innovations"),
        ..Default::default()
    };

    let mut state = ViewerGuiState::new();
    state.target_id = cli.id.unwrap_or_else(|| cli.connect.to_string());
    state.password = cli.password;
    state.frame_receiver = Some(frame_rx);
    state.input_sender = Some(input_tx);

    eframe::run_simple_native("AgentDesk Viewer", options, move |ctx, _frame| {
        state.render(ctx);
    })
    .map_err(|e| anyhow::anyhow!("Viewer GUI execution failed: {e}"))?;

    Ok(())
}
