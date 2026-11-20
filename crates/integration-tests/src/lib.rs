//! Integration test suite for AgentDesk.

#[cfg(test)]
mod tests {
    use agentdesk_capture::{CaptureConfig, MockCapturer, ScreenCapturer};
    use agentdesk_core::protocol::EncodedVideoPacket;
    use agentdesk_core::types::{InputEvent, ScreenSize};
    use agentdesk_encode::{MockDecoder, MockEncoder, VideoDecoder, VideoEncoder};
    use agentdesk_input::{InputInjector, MockInjector};
    use agentdesk_net::{TcpHostServer, TcpViewerClient};
    use std::net::SocketAddr;
    use std::time::Duration;
    use tokio::sync::{broadcast, mpsc};

    #[tokio::test]
    async fn test_end_to_end_frame_streaming_and_agent_api() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter("info")
            .try_init();

        // 1. Setup ports on localhost
        let tcp_port = 19091;
        let agent_api_port = 17879;
        let token = "test-secret-token-xyz".to_string();

        let tcp_addr: SocketAddr = format!("127.0.0.1:{tcp_port}").parse().unwrap();
        let api_addr: SocketAddr = format!("127.0.0.1:{agent_api_port}").parse().unwrap();

        let screen_size = ScreenSize {
            width: 800,
            height: 600,
        };

        let (video_broadcast_tx, _) = broadcast::channel::<EncodedVideoPacket>(32);
        let (input_event_tx, mut input_event_rx) = mpsc::channel::<InputEvent>(32);
        let (tcp_shutdown_tx, tcp_shutdown_rx) = mpsc::channel::<()>(1);
        let (api_shutdown_tx, api_shutdown_rx) = mpsc::channel::<()>(1);

        // 2. Start TCP Host Server
        let host_server = TcpHostServer::new(
            tcp_addr,
            token.clone(),
            screen_size,
            true, // plain TCP for fast test
            video_broadcast_tx.clone(),
            input_event_tx,
        );

        let host_task = tokio::spawn(async move {
            let _ = host_server.run(tcp_shutdown_rx).await;
        });

        // 3. Start Agent API Server
        let api_capturer = Box::new(MockCapturer::new(
            screen_size.width,
            screen_size.height,
            CaptureConfig::default(),
        ));
        let mock_injector = MockInjector::new();
        let injector_history = mock_injector.history.clone();
        let api_token = token.clone();

        let api_task = tokio::spawn(async move {
            let _ = agentdesk_agent_api::start_agent_api(
                api_addr,
                api_token,
                api_capturer,
                Box::new(mock_injector),
                api_shutdown_rx,
            )
            .await;
        });

        // Allow servers a moment to bind
        tokio::time::sleep(Duration::from_millis(150)).await;

        // 4. Test Agent API HTTP Endpoints using reqwest
        let client = reqwest::Client::new();

        // Test GET /health (unauthenticated)
        let health_url = format!("http://127.0.0.1:{agent_api_port}/health");
        let health_resp = client.get(&health_url).send().await.unwrap();
        assert_eq!(health_resp.status(), 200);
        let health_body: serde_json::Value = health_resp.json().await.unwrap();
        assert_eq!(health_body["status"], "ok");

        // Test POST /screenshot with Bearer auth
        let screenshot_url = format!("http://127.0.0.1:{agent_api_port}/screenshot");
        let shot_resp = client
            .post(&screenshot_url)
            .header("Authorization", format!("Bearer {token}"))
            .json(&serde_json::json!({}))
            .send()
            .await
            .unwrap();
        assert_eq!(shot_resp.status(), 200);
        let shot_body: serde_json::Value = shot_resp.json().await.unwrap();
        assert_eq!(shot_body["width"], 800);
        assert_eq!(shot_body["height"], 600);
        assert!(shot_body["png_base64"].as_str().unwrap().len() > 100);

        // Test POST /click
        let click_url = format!("http://127.0.0.1:{agent_api_port}/click");
        let click_resp = client
            .post(&click_url)
            .header("Authorization", format!("Bearer {token}"))
            .json(&serde_json::json!({ "x": 150.0, "y": 250.0, "button": "left", "clicks": 1 }))
            .send()
            .await
            .unwrap();
        assert_eq!(click_resp.status(), 200);

        // Verify injector recorded the click
        tokio::time::sleep(Duration::from_millis(50)).await;
        {
            let history = injector_history.lock().unwrap();
            assert!(!history.is_empty());
        }

        // Test Unauthorized on Agent API with wrong token
        let bad_auth_resp = client
            .post(&screenshot_url)
            .header("Authorization", "Bearer invalid-token")
            .json(&serde_json::json!({}))
            .send()
            .await
            .unwrap();
        assert_eq!(bad_auth_resp.status(), 401);

        // 5. Connect Viewer Client to Host over TCP
        let (handshake, mut viewer_video_rx, viewer_input_tx) =
            TcpViewerClient::connect(tcp_addr, token.clone(), true, None)
                .await
                .expect("Viewer connection failed");

        assert!(handshake.success);
        assert_eq!(handshake.screen.width, 800);
        assert_eq!(handshake.screen.height, 600);

        // 6. Send 10 frames from Host to Viewer
        let mut mock_capturer = MockCapturer::new(
            screen_size.width,
            screen_size.height,
            CaptureConfig::default(),
        );
        let mut mock_encoder = MockEncoder::new(screen_size.width, screen_size.height);
        let mut mock_decoder = MockDecoder::new();

        for _ in 0..10 {
            let frame = mock_capturer.capture_frame().unwrap();
            let packet = mock_encoder.encode_frame(&frame).unwrap();
            video_broadcast_tx.send(packet).unwrap();
        }

        // 7. Verify Viewer receives and decodes all 10 frames
        let mut received_frames = 0;
        let timeout = Duration::from_secs(3);
        let start = std::time::Instant::now();

        while received_frames < 10 && start.elapsed() < timeout {
            if let Ok(Some(packet)) =
                tokio::time::timeout(Duration::from_millis(500), viewer_video_rx.recv()).await
            {
                let decoded = mock_decoder.decode_packet(&packet).unwrap();
                assert_eq!(decoded.width, 800);
                assert_eq!(decoded.height, 600);
                received_frames += 1;
            }
        }
        assert_eq!(received_frames, 10, "Viewer must receive exactly 10 frames");

        // 8. Send input from viewer to host and verify host receives it
        viewer_input_tx
            .send(InputEvent::MouseMove { x: 300.0, y: 400.0 })
            .await
            .unwrap();

        let received_input =
            tokio::time::timeout(Duration::from_millis(500), input_event_rx.recv())
                .await
                .unwrap();
        assert_eq!(
            received_input,
            Some(InputEvent::MouseMove { x: 300.0, y: 400.0 })
        );

        // 9. Clean shutdown
        let _ = tcp_shutdown_tx.send(()).await;
        let _ = api_shutdown_tx.send(()).await;
        let _ = host_task.await;
        let _ = api_task.await;
    }
}
