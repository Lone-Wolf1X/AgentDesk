// AgentDesk — Free & Open Source Remote Desktop
// Copyright (c) 2025 Next Gen Innovations Private Limited
// Developer: Abhishek Kumar Paswan
// Website: https://nextgeninnovations.com.np/
// License: MIT — see LICENSE file
// Made in Nepal 🇳🇵

#[cfg(test)]
mod tests {
    use agentdesk_capture::{MockCapturer, ScreenCapturer};
    use agentdesk_core::config::CaptureConfig;
    use agentdesk_core::protocol::EncodedVideoPacket;
    use agentdesk_core::types::{InputEvent, ScreenSize};
    use agentdesk_encode::{MockDecoder, MockEncoder, VideoDecoder, VideoEncoder};
    use agentdesk_net::{TcpHostServer, TcpViewerClient};
    use std::net::SocketAddr;
    use std::time::Duration;
    use tokio::sync::{broadcast, mpsc};

    #[tokio::test]
    async fn test_end_to_end_remote_desktop_stream() {
        let _ = tracing_subscriber::fmt().with_env_filter("info").try_init();

        let tcp_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let password = "test-session-pass".to_string();

        let screen_size = ScreenSize {
            width: 1280,
            height: 720,
        };

        let (video_broadcast_tx, _) = broadcast::channel::<EncodedVideoPacket>(32);
        let (input_event_tx, mut input_event_rx) = mpsc::channel::<InputEvent>(32);
        let (tcp_shutdown_tx, tcp_shutdown_rx) = mpsc::channel::<()>(1);
        let (bound_tx, bound_rx) = tokio::sync::oneshot::channel::<SocketAddr>();

        // 1. Start Host TCP Server
        let host_server = TcpHostServer::new(
            tcp_addr,
            password.clone(),
            screen_size,
            true, // plain TCP for fast local test
            video_broadcast_tx.clone(),
            input_event_tx,
        );

        let host_task = tokio::spawn(async move {
            let _ = host_server.run_with_bound_addr(tcp_shutdown_rx, Some(bound_tx)).await;
        });

        let actual_addr = bound_rx.await.expect("Failed to get bound addr");

        // 2. Connect Viewer Client
        let (handshake, mut viewer_video_rx, viewer_input_tx) =
            TcpViewerClient::connect(actual_addr, password.clone(), true, None)
                .await
                .expect("Viewer failed to connect to host");

        assert!(handshake.success);
        assert_eq!(handshake.screen.width, 1280);
        assert_eq!(handshake.screen.height, 720);

        // 3. Stream 10 frames from Host to Viewer
        let mut capturer = MockCapturer::new(1280, 720, CaptureConfig::default());
        let mut encoder = MockEncoder::new(1280, 720);
        let mut decoder = MockDecoder::new();

        for _ in 0..10 {
            let frame = capturer.capture_frame().unwrap();
            let packet = encoder.encode_frame(&frame).unwrap();
            video_broadcast_tx.send(packet).unwrap();
            tokio::time::sleep(Duration::from_millis(25)).await;
        }

        // 4. Verify Viewer decodes all 10 frames
        let mut received = 0;
        let start = std::time::Instant::now();
        while received < 10 && start.elapsed() < Duration::from_secs(3) {
            if let Ok(Some(packet)) =
                tokio::time::timeout(Duration::from_millis(500), viewer_video_rx.recv()).await
            {
                let frame = decoder.decode_packet(&packet).unwrap();
                assert_eq!(frame.width, 1280);
                assert_eq!(frame.height, 720);
                received += 1;
            }
        }
        assert_eq!(received, 10);

        // 5. Send input from viewer to host
        viewer_input_tx
            .send(InputEvent::MouseMove { x: 500.0, y: 300.0 })
            .await
            .unwrap();

        let host_input = tokio::time::timeout(Duration::from_millis(500), input_event_rx.recv())
            .await
            .unwrap();
        assert_eq!(host_input, Some(InputEvent::MouseMove { x: 500.0, y: 300.0 }));

        // 6. Graceful shutdown
        let _ = tcp_shutdown_tx.send(()).await;
        let _ = host_task.await;
    }
}
