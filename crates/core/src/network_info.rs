// AgentDesk — Free & Open Source Remote Desktop
// Copyright (c) 2025 Next Gen Innovations Private Limited
// Developer: Abhishek Kumar Paswan
// Website: https://nextgeninnovations.com.np/
// License: MIT — see LICENSE file
// Made in Nepal 🇳🇵

use std::net::IpAddr;

/// Returns the primary local IP address (non-loopback, non-link-local).
pub fn get_local_ip() -> Option<IpAddr> {
    use std::net::UdpSocket;
    // Trick: connect to a public IP (UDP connect does not send packets over the network)
    // to allow the OS kernel routing table to determine the outbound network interface.
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip())
}

/// Returns a list of local IP addresses.
pub fn get_all_local_ips() -> Vec<IpAddr> {
    let mut ips = Vec::new();
    if let Some(primary) = get_local_ip() {
        ips.push(primary);
    }
    ips
}
