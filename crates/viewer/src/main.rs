//! AgentDesk Viewer Binary (`agentdesk-viewer`)
//!
//! Connects to an AgentDesk host, performs authenticated handshake, receives
//! H.264 encoded video stream, decodes frames with OpenH264, and optionally saves
//! frames to disk for verification or display.

use agentdesk_core::types::InputEvent;
use agentdesk_encode::{OpenH264Decoder, VideoDecoder};
use agentdesk_net::TcpViewerClient;
use anyhow::{Context, Result};
use clap::Parser;
use image::{ImageBuffer, ImageFormat, Rgba};
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Instant;
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(name = "agentdesk-viewer", author, version, about = "AgentDesk Viewer Client")]
struct Cli {
    /// Remote host address to connect to
    #[arg(short, long, default_value = "127.0.0.1:9090")]
    connect: SocketAddr,

    /// Authentication token
    #[arg(short, long, env = "AGENTDESK_TOKEN", default_value = "agentdesk-secret-token")]
    token: String,

    /// Optional directory to save received decoded frames as PNG images
    #[arg(long)]
    save_frames: Option<PathBuf>,

    /// Maximum frames to receive before exiting (useful for automated testing)
    #[arg(long)]
    max_frames: Option<u64>,

    /// Disable TLS (connect via plain TCP)
    #[arg(long, default_value_t = false)]
    no_tls: bool,

    /// Pinned SHA-256 TLS certificate fingerprint of the host
    #[arg(long)]
    tls_fingerprint: Option<String>,

    /// Run with GUI window (Phase 2 stub)
    #[arg(long, default_value_t = false)]
    windowed: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,agentdesk_viewer=debug,agentdesk_net=debug".into()),
        )
        .init();

    let cli = Cli::parse();
    info!("Starting AgentDesk Viewer v{}", env!("CARGO_PKG_VERSION"));

    if let Some(ref save_dir) = cli.save_frames {
        fs::create_dir_all(save_dir)
            .with_context(|| format!("Failed to create save directory: {:?}", save_dir))?;
        info!("Saving received decoded frames to: {:?}", save_dir);
    }

    if cli.windowed {
        info!("Windowed GUI rendering is slated for Phase 2 (winit/pixels integration).");
    }

    info!("Connecting to host at {}...", cli.connect);
    let (handshake_resp, mut video_rx, _input_tx) = TcpViewerClient::connect(
        cli.connect,
        cli.token,
        cli.no_tls,
        cli.tls_fingerprint,
    )
    .await
    .context("Failed to connect and authenticate with host")?;

    info!(
        "Connected to host v{}! Remote screen dimensions: {}x{}",
        handshake_resp.host_version,
        handshake_resp.screen.width,
        handshake_resp.screen.height
    );

    let mut decoder: Box<dyn VideoDecoder> = match OpenH264Decoder::new() {
        Ok(dec) => Box::new(dec),
        Err(e) => {
            warn!("OpenH264 decoder init failed ({e}), using mock decoder");
            Box::new(agentdesk_encode::MockDecoder::new())
        }
    };

    let mut frame_count = 0u64;
    let start_time = Instant::now();

    while let Some(packet) = video_rx.recv().await {
        frame_count += 1;

        match decoder.decode_packet(&packet) {
            Ok(frame) => {
                if frame_count % 20 == 0 || frame_count == 1 {
                    let elapsed = start_time.elapsed().as_secs_f64();
                    let current_fps = if elapsed > 0.0 {
                        frame_count as f64 / elapsed
                    } else {
                        0.0
                    };

                    info!(
                        "Received frame #{frame_count} ({}x{}, keyframe={}, {:.1} fps)",
                        frame.width, frame.height, packet.is_keyframe, current_fps
                    );
                }

                // Save frame to disk if requested
                if let Some(ref save_dir) = cli.save_frames {
                    let filename = save_dir.join(format!("frame_{:05}.png", frame_count));
                    let rgba = frame.to_rgba();
                    if let Some(img) = ImageBuffer::<Rgba<u8>, _>::from_raw(frame.width, frame.height, rgba) {
                        let _ = img.save_with_format(&filename, ImageFormat::Png);
                    }
                }
            }
            Err(e) => {
                // Some initial packets are SPS/PPS headers that don't produce frames immediately
                tracing::debug!("Decode note on frame #{frame_count}: {e}");
            }
        }

        if let Some(max) = cli.max_frames {
            if frame_count >= max {
                info!("Reached maximum requested frames ({max}). Exiting successfully.");
                break;
            }
        }
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    info!(
        "Viewer session finished: received {frame_count} frames in {:.2} seconds ({:.1} avg fps)",
        elapsed,
        if elapsed > 0.0 {
            frame_count as f64 / elapsed
        } else {
            0.0
        }
    );

    Ok(())
}
