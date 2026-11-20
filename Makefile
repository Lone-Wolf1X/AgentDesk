.PHONY: all build test clean run-host run-viewer run-signal fmt clippy

# Default target
all: build

# Build all workspace crates in debug mode
build:
	cargo build --workspace

# Build all workspace crates in release mode
build-release:
	cargo build --workspace --release

# Run all unit and integration tests across the workspace
test:
	cargo test --workspace -- --nocapture

# Format all code
fmt:
	cargo fmt --all

# Run clippy with strict warnings
clippy:
	cargo clippy --workspace --all-targets -- -D warnings

# Clean build artifacts
clean:
	cargo clean

# Run agentdesk-host with default local testing settings
run-host:
	cargo run -p agentdesk-host -- --listen 127.0.0.1:9090 --token test-token --no-tls

# Run agentdesk-host with mock display (safe in headless / CI environments)
run-host-mock:
	cargo run -p agentdesk-host -- --listen 127.0.0.1:9090 --token test-token --mock-display --no-tls

# Run agentdesk-viewer connecting to local host
run-viewer:
	cargo run -p agentdesk-viewer -- --connect 127.0.0.1:9090 --token test-token --no-tls

# Run agentdesk-signal WebRTC signaling server
run-signal:
	cargo run -p agentdesk-signal -- --listen 127.0.0.1:8080
