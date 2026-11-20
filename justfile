# AgentDesk Justfile (modern command runner)

# Default: build workspace
default: build

# Build workspace in debug mode
build:
    cargo build --workspace

# Build workspace in release mode
build-release:
    cargo build --workspace --release

# Run all workspace unit and integration tests
test:
    cargo test --workspace -- --nocapture

# Check code formatting
fmt-check:
    cargo fmt --all -- --check

# Format all Rust files
fmt:
    cargo fmt --all

# Run clippy linter with strict warning deny
clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# Run host in standard mode
run-host:
    cargo run -p agentdesk-host -- --listen 127.0.0.1:9090 --token test-token --no-tls

# Run host in mock display mode (for headless environments / VMs)
run-host-mock:
    cargo run -p agentdesk-host -- --listen 127.0.0.1:9090 --token test-token --mock-display --no-tls

# Run viewer client
run-viewer:
    cargo run -p agentdesk-viewer -- --connect 127.0.0.1:9090 --token test-token --no-tls

# Run signaling server
run-signal:
    cargo run -p agentdesk-signal -- --listen 127.0.0.1:8080
