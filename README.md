# AgentDesk

Free and open-source remote desktop application developed by Next Gen Innovations Private Limited.

## Features
- Direct low-latency screen streaming with H.264 compression.
- 9-digit session connection ID.
- Cross-platform support for Windows, Linux (X11), and macOS.
- Built-in graphical user interface for host and viewer.
- Headless execution mode for server administration.
- No usage limits, time limits, or external account requirements.

## Workspace Architecture
- `crates/core`: Core types, protocol framing, and security primitives.
- `crates/capture`: Screen capture abstraction.
- `crates/input`: Keyboard and mouse simulation.
- `crates/encode`: Video encoding and decoding.
- `crates/net`: Length-prefixed TCP streaming with TLS.
- `crates/gui`: Graphical interface components.
- `crates/host`: Host server application.
- `crates/viewer`: Viewer client application.
- `signal-server`: ID registration and brokering server.

## Getting Started

### Building from Source
```bash
cargo build --workspace
```

### Running Host
```bash
cargo run -p agentdesk-host
```

### Running Viewer
```bash
cargo run -p agentdesk-viewer -- --connect 127.0.0.1:9090
```

Refer to [USER_GUIDE.md](file:///d:/AgentDesk/USER_GUIDE.md) for detailed operational instructions.
