// AgentDesk — Local Peer Discovery via UDP Broadcast
// Copyright (c) 2025 Next Gen Innovations Private Limited
// Developer: Abhishek Kumar Paswan
// Website: https://nextgeninnovations.com.np/
// License: MIT
// Made in Nepal 🇳🇵

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tracing::{debug, info};

pub const DISCOVERY_PORT: u16 = 9098;

#[derive(Clone)]
pub struct PeerDiscovery {
    pub my_id: String,
    pub my_port: u16,
    pub peers: Arc<RwLock<HashMap<String, SocketAddr>>>,
}

impl PeerDiscovery {
    pub fn new(my_id: String, my_port: u16) -> Self {
        Self {
            my_id,
            my_port,
            peers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start background listener and periodic beacon
    pub async fn start(&self) -> Arc<RwLock<HashMap<String, SocketAddr>>> {
        let peers = self.peers.clone();
        let my_id = self.my_id.clone();
        let my_port = self.my_port;

        // Try binding to 0.0.0.0:9098, or ephemeral port if 9098 is in use
        let socket = match UdpSocket::bind(format!("0.0.0.0:{DISCOVERY_PORT}")).await {
            Ok(s) => {
                let _ = s.set_broadcast(true);
                info!("PeerDiscovery: Listening on broadcast port {DISCOVERY_PORT}");
                Arc::new(s)
            }
            Err(_) => {
                let s = UdpSocket::bind("0.0.0.0:0").await.expect("Failed to bind UDP ephemeral socket");
                let _ = s.set_broadcast(true);
                info!("PeerDiscovery: Bound to ephemeral UDP port (broadcast port in use)");
                Arc::new(s)
            }
        };

        // Task 1: Periodic broadcast beacon (every 1.5 seconds)
        let sock_send = socket.clone();
        let b_id = my_id.clone();
        tokio::spawn(async move {
            let msg = format!("AGENTDESK:BEACON:{b_id}:{my_port}");
            let broadcast_addrs = [
                format!("255.255.255.255:{DISCOVERY_PORT}"),
                format!("127.0.0.1:{DISCOVERY_PORT}"),
            ];
            loop {
                for addr in &broadcast_addrs {
                    let _ = sock_send.send_to(msg.as_bytes(), addr).await;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
            }
        });

        // Task 2: Listen for incoming beacons, queries, and replies
        let sock_recv = socket.clone();
        let p_map = peers.clone();
        let local_id = my_id.clone();
        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            loop {
                match sock_recv.recv_from(&mut buf).await {
                    Ok((len, peer_addr)) => {
                        if let Ok(text) = std::str::from_utf8(&buf[..len]) {
                            let text = text.trim();
                            if text.starts_with("AGENTDESK:BEACON:") {
                                // Format: AGENTDESK:BEACON:<id>:<port>
                                let parts: Vec<&str> = text.split(':').collect();
                                if parts.len() >= 4 {
                                    let id = parts[2].to_string();
                                    if let Ok(port) = parts[3].parse::<u16>() {
                                        if id != local_id {
                                            let host_addr = SocketAddr::new(peer_addr.ip(), port);
                                            debug!("Discovered LAN peer {id} at {host_addr}");
                                            p_map.write().await.insert(id, host_addr);
                                        }
                                    }
                                }
                            } else if text.starts_with("AGENTDESK:QUERY:") {
                                // Format: AGENTDESK:QUERY:<id>
                                let parts: Vec<&str> = text.split(':').collect();
                                if parts.len() >= 3 && parts[2] == local_id {
                                    debug!("Responding to query for our ID {local_id}");
                                    let reply = format!("AGENTDESK:REPLY:{local_id}:{my_port}");
                                    let _ = sock_recv.send_to(reply.as_bytes(), peer_addr).await;
                                }
                            } else if text.starts_with("AGENTDESK:REPLY:") {
                                // Format: AGENTDESK:REPLY:<id>:<port>
                                let parts: Vec<&str> = text.split(':').collect();
                                if parts.len() >= 4 {
                                    let id = parts[2].to_string();
                                    if let Ok(port) = parts[3].parse::<u16>() {
                                        let host_addr = SocketAddr::new(peer_addr.ip(), port);
                                        debug!("Received reply for peer {id} at {host_addr}");
                                        p_map.write().await.insert(id, host_addr);
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }
            }
        });

        peers
    }

    /// Resolve a 9-digit ID into a SocketAddr (checks local cache first, then active query)
    pub async fn resolve_id(
        peers: &Arc<RwLock<HashMap<String, SocketAddr>>>,
        target_id: &str,
        timeout_ms: u64,
    ) -> Option<SocketAddr> {
        let clean_id = target_id.replace([' ', '-'], "");

        // 1. Check in-memory cache first
        if let Some(addr) = peers.read().await.get(&clean_id) {
            return Some(*addr);
        }

        // 2. Broadcast active query if not found in cache
        if let Ok(query_sock) = UdpSocket::bind("0.0.0.0:0").await {
            let _ = query_sock.set_broadcast(true);
            let msg = format!("AGENTDESK:QUERY:{clean_id}");
            let broadcast_addrs = [
                format!("255.255.255.255:{DISCOVERY_PORT}"),
                format!("127.0.0.1:{DISCOVERY_PORT}"),
            ];
            for addr in &broadcast_addrs {
                let _ = query_sock.send_to(msg.as_bytes(), addr).await;
            }

            // Wait for response up to timeout_ms
            let start = tokio::time::Instant::now();
            let mut buf = [0u8; 1024];
            while start.elapsed().as_millis() < timeout_ms as u128 {
                let remaining = (timeout_ms as u128).saturating_sub(start.elapsed().as_millis());
                if remaining == 0 {
                    break;
                }
                match tokio::time::timeout(
                    tokio::time::Duration::from_millis(remaining as u64),
                    query_sock.recv_from(&mut buf),
                )
                .await
                {
                    Ok(Ok((len, peer_addr))) => {
                        if let Ok(text) = std::str::from_utf8(&buf[..len]) {
                            let text = text.trim();
                            if text.starts_with("AGENTDESK:REPLY:") {
                                let parts: Vec<&str> = text.split(':').collect();
                                if parts.len() >= 4 && parts[2] == clean_id {
                                    if let Ok(port) = parts[3].parse::<u16>() {
                                        let addr = SocketAddr::new(peer_addr.ip(), port);
                                        peers.write().await.insert(clean_id.clone(), addr);
                                        return Some(addr);
                                    }
                                }
                            }
                        }
                    }
                    _ => break,
                }
            }
        }

        // Final cache check
        peers.read().await.get(&clean_id).copied()
    }
}
