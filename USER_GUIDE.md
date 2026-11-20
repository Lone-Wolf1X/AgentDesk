# AgentDesk User Manual

## Overview
AgentDesk provides direct remote desktop control between machines using screen streaming and input relay.

### Running the Host (Controlled Device)
Launch the host application to allow remote connections:
```bash
cargo run -p agentdesk-host
```
Features available in the Host window:
- **Connection ID**: Unique 9-digit identifier to share with the viewer.
- **Access Password**: Password required by connecting viewers.
- **Session Controls**: Terminate connection or pause remote input anytime.

### Headless Host Mode (Servers)
To run on remote servers without a display:
```bash
cargo run -p agentdesk-host -- --headless --token "your-password"
```

### Running the Viewer (Controlling Device)
Launch the viewer application:
```bash
cargo run -p agentdesk-viewer
```
Enter the 9-digit Connection ID or the direct IP address along with the host password to start viewing and interacting with the remote screen.
