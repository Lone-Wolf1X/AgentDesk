# AgentDesk 🇳🇵

> **Free & Open Source Remote Desktop — Made in Nepal**  
> Developed by **Next Gen Innovations Private Limited** & **Abhishek Kumar Paswan**  
> Website: [https://nextgeninnovations.com.np/](https://nextgeninnovations.com.np/)

AgentDesk is a high-performance, lightweight, and 100% free remote desktop application built in Rust — designed as an open-source alternative to AnyDesk and TeamViewer.

---

## 🚀 Single App, Both Roles (Unified Architecture)

AgentDesk is **one single app** that can both **host** and **view** remote desktop sessions.

- **One download, one install**: No separate "host edition" or "viewer edition".
- **Zero Configuration**: Simply open the app:
  - **This Device**: Instantly displays your local IP (`192.168.1.15:9090`), connection ID, and auto-generated secure password.
  - **Connect to Remote**: Enter the remote device's IP and password, then click Connect.
- **Direct LAN & WAN**: Direct IP:Port connections with sub-millisecond latency.
- **100% Free & Open Source**: No telemetry, no time limits, no device limits, and no accounts required.

---

## 📥 Downloads

Pre-built binaries are available with each release:
- **[GitHub Releases](https://github.com/Lone-Wolf1X/AgentDesk/releases)**: Windows `.zip`, macOS `.tar.gz`, and Linux `.tar.gz`.
- Or easily [build from source](docs/BUILD_DISTRIBUTION.md).

## 🧪 Testing with Two Real Machines

Follow our step-by-step Hindi-English testing guide to connect two machines across your local network:
- 📖 **[Two-PC Testing Guide](docs/TESTING_GUIDE.md)**

## 📦 Build Distributables & Packaging

Learn how to compile standalone executables and native installers:
- **[Windows .exe / Inno Setup Installer](docs/BUILD_DISTRIBUTION.md#a-windows--exe--installer-build)**
- **[macOS Universal Binary & .dmg](docs/BUILD_DISTRIBUTION.md#b-macos--universal-binary--dmg-build)**
- **[Linux .AppImage / .deb](docs/BUILD_DISTRIBUTION.md#c-linux--appimage--deb-build)**
- **[Binary Size & Optimization Note](docs/BINARY_SIZES_NOTE.md)**

---

## 🏗️ Workspace Architecture

```
AgentDesk/
├── Cargo.toml                  # Cargo workspace & optimized release profile
├── .github/workflows/release.yml # Multi-platform CI/CD release workflow
├── packaging/                  # Native OS packaging templates
│   ├── windows/installer.iss
│   ├── macos/Info.plist
│   └── linux/agentdesk.desktop
├── docs/                       # Comprehensive guides
│   ├── TESTING_GUIDE.md
│   ├── BUILD_DISTRIBUTION.md
│   └── BINARY_SIZES_NOTE.md
└── crates/
    ├── core/                   # Protocol packets, framing, encryption, network_info
    ├── capture/                # Screen capture abstraction (scap & DXGI)
    ├── input/                  # Native input simulation & dispatch (rdev)
    ├── encode/                 # H.264 software codec (OpenH264)
    ├── net/                    # Length-prefixed TCP streaming, TLS, available port binder
    ├── gui/                    # Unified dual-role egui interface (This Device + Connect)
    ├── app/                    # Unified `agentdesk` binary
    ├── integration-tests/      # Dual-role and streaming test suites
    └── signal-server/          # Connection ID brokering server
```

---

## ⚡ Quick Start

### 1. Build
```bash
cargo build --release --workspace
```

### 2. Run GUI (Standard Use)
Simply run the binary on any machine:
```bash
# Windows
.\target\release\agentdesk.exe

# macOS / Linux
./target/release/agentdesk
```

### 3. Advanced CLI Usage
```bash
# Run headless server
./agentdesk host --listen 0.0.0.0:9090 --token "mysecretpass" --headless

# Quick connect directly to a remote host
./agentdesk connect --connect 192.168.1.50:9090 --password "mysecretpass"
```

---

## 📄 License

MIT License — Copyright (c) 2025 Next Gen Innovations Private Limited and Abhishek Kumar Paswan.
