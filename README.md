# AgentDesk 🇳🇵

> **Free & Open Source Remote Desktop — Made in Nepal**  
> Developed by **Next Gen Innovations Private Limited** & **Abhishek Kumar Paswan**  
> Website: [https://nextgeninnovations.com.np/](https://nextgeninnovations.com.np/)

AgentDesk is a high-performance, lightweight, and completely free alternative to AnyDesk and TeamViewer built in Rust. It provides direct low-latency screen streaming and bidirectional keyboard/mouse remote control without telemetry, time limits, or account requirements.

---

## 📥 Downloads

Pre-built binaries are published with each release:
- **[GitHub Releases](https://github.com/Lone-Wolf1X/AgentDesk/releases)**: Download Windows `.zip`, macOS `.tar.gz`, and Linux `.tar.gz`.
- Until pre-built binaries are downloaded, you can easily [build from source](docs/BUILD_DISTRIBUTION.md).

## 🧪 Testing with Two PCs

Follow our step-by-step Hindi-English testing guide to connect two real machines across your local network:
- 📖 **[Two-PC Testing Guide](docs/TESTING_GUIDE.md)**

## 📦 Build Distributables & Packaging

Learn how to compile standalone executables and native installers:
- **[Windows .exe / Inno Setup Installer](docs/BUILD_DISTRIBUTION.md#a-windows--exe--installer-build)**
- **[macOS Universal Binary & .dmg](docs/BUILD_DISTRIBUTION.md#b-macos--universal-binary--dmg-build)**
- **[Linux .AppImage / .deb / .rpm](docs/BUILD_DISTRIBUTION.md#c-linux--appimage-deb--rpm-build)**
- **[Binary Size & Optimization Note](docs/BINARY_SIZES_NOTE.md)**

---

## 🚀 Key Features

- **Zero Cost & Open Source**: MIT Licensed with no paywalls, session limits, or device counters.
- **H.264 Video Streaming**: Realtime screen capture compressed with OpenH264 for smooth 30-60 FPS display.
- **9-Digit Connection ID**: Easy AnyDesk-like connection brokering via signal server or direct IP.
- **TLS Security**: End-to-end encrypted transport with fingerprint verification.
- **Interactive Host Controls**: Interactive accept/reject connection prompts and immediate access revocation.
- **Cross-Platform**: First-class support for Windows (DXGI), macOS (AVFoundation), and Linux (X11).
- **Headless Mode**: Dedicated `--headless` flag for remote server administration.

---

## 🏗️ Workspace Architecture

```
AgentDesk/
├── Cargo.toml                  # Cargo workspace & optimized release profiles
├── .github/workflows/          # Automated multi-platform CI/CD release workflow
├── packaging/                  # Native OS packaging templates (Inno Setup, Plist, Desktop)
│   ├── windows/installer.iss
│   ├── macos/Info.plist
│   └── linux/agentdesk.desktop
├── docs/                       # Comprehensive documentation & testing guides
│   ├── TESTING_GUIDE.md
│   ├── BUILD_DISTRIBUTION.md
│   └── BINARY_SIZES_NOTE.md
└── crates/
    ├── core/                   # Protocol packets, framing, encryption & handshake
    ├── capture/                # Screen capture abstraction (scap & DXGI)
    ├── input/                  # Native input simulation & interception (rdev)
    ├── encode/                 # H.264 software codec (OpenH264)
    ├── net/                    # Length-prefixed TCP streaming & TLS
    ├── gui/                    # Shared egui / eframe UI components
    ├── host/                   # agentdesk-host executable
    ├── viewer/                 # agentdesk-viewer executable
    ├── integration-tests/      # End-to-end streaming & input test suite
    └── signal-server/          # Connection ID brokering server
```

---

## ⚡ Quick Start

### 1. Build from Source
```bash
cargo build --release --workspace
```

### 2. Start Host (Target Machine)
```bash
# GUI mode
cargo run --release -p agentdesk-host

# Headless mode with custom token
cargo run --release -p agentdesk-host -- --headless --token "mysecretpass"
```

### 3. Start Viewer (Client Machine)
```bash
# Connect using 9-digit Connection ID
cargo run --release -p agentdesk-viewer -- --id 123456789 --password "mysecretpass"

# Connect directly via IP
cargo run --release -p agentdesk-viewer -- --connect 192.168.1.10:9090 --password "mysecretpass"
```

---

## 📄 License

MIT License — Copyright (c) 2025 Next Gen Innovations Private Limited and Abhishek Kumar Paswan.
