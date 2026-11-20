# 🧪 AgentDesk Testing Guide: Do Real PCs Par Testing Kaise Karein

Yeh guide aapko step-by-step batati hai ki kaise do alag-alag physical machines (PCs / Laptops) par **AgentDesk** ko run aur test karna hai — jaise AnyDesk ya TeamViewer use karte hain.

---

## 📌 Testing Overview

```
+--------------------------+                         +--------------------------+
|     PC-1: HOST (Target)   |                         |   PC-2: VIEWER (Client)  |
|  (Jiski screen dekhni &  | <=== H.264 Video Stream |  (Jo baith kar control   |
|   control karni hai)     | ===> Keyboard/Mouse === |   karega)                |
|                          |     (Port 9090 / TCP)   |                          |
+--------------------------+                         +--------------------------+
```

---

## A. Prerequisites (Dono PCs Ke Liye)

Testing shuru karne se pehle dono machines par basic requirements check karein:

### 1. Windows Machine
- **C++ Build Tools**: Visual Studio 2022 Build Tools (Desktop development with C++).
- **Rust Stable (MSVC)**:
  ```powershell
  rustup default stable-x86_64-pc-windows-msvc
  ```

### 2. macOS Machine
- **Xcode Command Line Tools**:
  ```bash
  xcode-select --install
  ```
- **System Permissions**:
  - `System Settings` -> `Privacy & Security` -> `Screen Recording` -> Terminal ya AgentDesk ko **Allow** karein.
  - `System Settings` -> `Privacy & Security` -> `Accessibility` -> Terminal ya AgentDesk ko **Allow** karein (keyboard/mouse simulation ke liye zaroori hai).

### 3. Linux Machine
- **Display Server**: X11 session zaroori hai (**Wayland abhi experimental/unsupported hai**; login screen par gear icon se "GNOME on Xorg" ya "X11" select karein).
- **Required Packages**:
  - **Ubuntu / Debian**:
    ```bash
    sudo apt update
    sudo apt install -y build-essential pkg-config libx11-dev libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
    ```
  - **Arch Linux / Manjaro**:
    ```bash
    sudo pacman -S --needed base-devel libx11 libxcb libxkbcommon openssl
    ```

### 4. Network Setup
- Shuruat ke liye **dono PCs same WiFi router ya LAN network se connected hon** (Phase 1 direct connection testing).

---

## B. Step 1 — Build on Both Machines

Dono machines par repository clone karein aur release binaries build karein:

```bash
git clone https://github.com/Lone-Wolf1X/AgentDesk.git
cd AgentDesk
cargo build --release --workspace
```

> **Debug vs Release Build**:
> Debug build (`cargo build`) bina optimization ke banta hai aur slow screen capture / video encoding de sakta hai. Release build (`cargo build --release`) full LTO aur compiler optimization use karta hai, jisse 30-60 FPS smooth video streaming milti hai.

### Compiled Binaries Ki Location:
- **Windows**:
  - `target\release\agentdesk-host.exe`
  - `target\release\agentdesk-viewer.exe`
  - `target\release\agentdesk-signal.exe`
- **macOS / Linux**:
  - `target/release/agentdesk-host`
  - `target/release/agentdesk-viewer`
  - `target/release/agentdesk-signal`

---

## C. Step 2 — Start Host (PC-1 Par)

Jis PC ko remote control karna hai, uspar `agentdesk-host` run karein:

```powershell
# Windows PowerShell
.\target\release\agentdesk-host.exe --token test123 --fps 25 --bitrate 2500
```
```bash
# macOS / Linux Terminal
./target/release/agentdesk-host --token test123 --fps 25 --bitrate 2500
```

### Screen Par Kya Dikhega:
1. **AgentDesk Host GUI Window** open hogi:
   - Green badge: **"Status: Waiting for connection"**
   - **Your Connection ID**: Auto-generated 9-digit ID (jaise `482-195-730`)
   - **Password**: `test123` (agar aap `--token` flag nahi denge, toh random secure password generate hoga aur screen par dikhega)
2. **Firewall Prompt (Windows/macOS)**:
   - Windows Defender Firewall ka prompt aayega: "Allow AgentDesk Host to communicate on Private networks".
   - **"Allow access"** par click karein.
   - ![Firewall Prompt Placeholder](images/firewall_prompt.png)

---

## D. Step 3 — Start Viewer (PC-2 Par)

Controlling PC par `agentdesk-viewer` run karein:

### Option 1: Direct IP Mode (LAN par sabse reliable)
Pehle Host PC ka local IP address pata karein:
- **Windows Host PC**: Terminal mein `ipconfig` run karein -> `IPv4 Address` dekhein (e.g. `192.168.1.15`).
- **macOS / Linux Host PC**: Terminal mein `ip addr` ya `ifconfig` run karein (e.g. `192.168.1.15`).

Ab Viewer PC par command run karein:
```powershell
# Windows
.\target\release\agentdesk-viewer.exe --connect 192.168.1.15:9090 --password test123
```
```bash
# macOS / Linux
./target/release/agentdesk-viewer --connect 192.168.1.15:9090 --password test123
```

### Option 2: 9-Digit ID Mode (via Signal Broker)
Agar local signal broker chal raha hai:
```bash
./target/release/agentdesk-viewer --id 482195730 --password test123
```

---

## E. Step 4 — Connection Accept Karna & TLS Trust

1. **Host PC par notification popup aayega**:
   - `"Incoming connection from 192.168.1.24. Allow viewer to view and control your desktop?"`
   - **[Accept]** button par click karein.
2. **Viewer PC par TLS Trust prompt**:
   - Pehli baar connect karne par host ka unique self-signed TLS fingerprint verify hota hai.
   - **"Trust Host Fingerprint"** confirm karein. Isse end-to-end encryption authenticate ho jati hai.
3. Ab session live ho jayega!

---

## F. Step 5 — Verification Checklist

Dono PCs connect hone ke baad yeh tests perform karein:

| Test | Expected Result | Pass / Fail |
|---|---|---|
| **Remote Screen Visibility** | Viewer window ke andar Host PC ka exact desktop realtime dikhna chahiye. | `[  ]` |
| **Mouse Cursor Movement** | Viewer window mein mouse move karein -> Host PC par cursor sath-sath move hona chahiye. | `[  ]` |
| **Mouse Click & Double Click** | Viewer mein desktop icon ya folder par click karein -> Host par open hona chahiye. | `[  ]` |
| **Keyboard Typing** | Host par Notepad / Text Editor kholein, Viewer se type karein -> Host par text print hona chahiye. | `[  ]` |
| **Keyboard Shortcuts** | `Ctrl+C`, `Ctrl+V`, ya `Alt+Tab` Viewer se press karein -> Host par trigger hona chahiye. | `[  ]` |
| **FPS & Stream Stability** | 10 minute tak continuous use karein -> video frame freeze ya disconnect nahi hona chahiye. | `[  ]` |
| **Clean Disconnect** | Viewer window close karein -> Host screen par status wapas **"Waiting for connection"** show hona chahiye. | `[  ]` |

---

## G. Troubleshooting (Common Issues & Solutions)

### 1. "Connection refused" ya "Timed out"
- **Reason**: Host PC ka firewall port `9090` block kar raha hai.
- **Fix (Windows Host)**:
  ```powershell
  # Administrator PowerShell mein run karein
  New-NetFirewallRule -DisplayName "AgentDesk Host Port" -Direction Inbound -LocalPort 9090 -Protocol TCP -Action Allow
  ```
- **Fix (Linux Host)**:
  ```bash
  sudo ufw allow 9090/tcp
  ```

### 2. "TLS fingerprint mismatch"
- Agar host ka certificate re-generate hua ho ya IP conflict ho, fingerprint update karne ke liye prompt mein "Update Trust" select karein. (Agar unexpected machine ho toh security warning samjhein).

### 3. Black Screen on Viewer
- **macOS Host**: `System Settings` -> `Privacy & Security` -> `Screen Recording` mein terminal / binary ko permission granted nahi hai. Permission dekar application restart karein.
- **Linux Host**: Host system Wayland session par chal raha hai. Log out karein aur login screen par gear icon se "X11 / Xorg" select karein.

### 4. Input (Mouse/Keyboard) Control Nahi Ho Raha
- **macOS Host**: `System Settings` -> `Privacy & Security` -> `Accessibility` permission missing hai.
- **Windows Host**: Agar host par koi window "Run as Administrator" khuli ho, toh background input inject karne ke liye `agentdesk-host.exe` ko bhi "Run as Administrator" start karna hoga.

### 5. Video Lag ya High Latency
- Dono machines 5 GHz WiFi ya Ethernet LAN cable se connect karein.
- Bitrate aur FPS adjust karein:
  ```bash
  ./agentdesk-host --fps 20 --bitrate 1200
  ```

---

## H. Network Testing Scenarios

1. **Scenario 1: Same Local LAN (Baseline)**
   - Dono machines same domestic/office WiFi router par hain.
   - Lowest latency (< 15ms) aur maximum picture quality test karne ke liye perfect hai.
2. **Scenario 2: Different Subnets / Mobile Hotspot**
   - Host PC home WiFi par ho aur Viewer PC mobile 4G/5G hotspot par ho.
   - Signal server ke through broker connection test karne ke liye use hota hai.
3. **Scenario 3: Direct Internet Over Port Forward**
   - Agar Host router mein public IP available hai, toh router admin page mein Port `9090` (TCP) ko Host machine ke local IP par forward karein.
   - Viewer kahin se bhi connect kar sakta hai:
     ```bash
     ./agentdesk-viewer --connect <PUBLIC_IP>:9090 --password test123
     ```

---

## I. Reporting Bugs & Logs

Agar koi issue ya crash aaye, toh detailed debug logs ke sath issue report karein:

1. **Run with Verbose Debug Logs**:
   ```bash
   # Windows PowerShell
   $env:RUST_LOG="debug,agentdesk=trace"
   .\target\release\agentdesk-host.exe

   # Linux / macOS
   RUST_LOG="debug,agentdesk=trace" ./target/release/agentdesk-host
   ```
2. **GitHub Issue Open Karein**:
   - Link: [AgentDesk Issues](https://github.com/Lone-Wolf1X/AgentDesk/issues)
   - **Template**:
     - **Host OS & Version**: (e.g. Windows 11 23H2 / Ubuntu 22.04)
     - **Viewer OS & Version**: (e.g. macOS Sonoma 14.5)
     - **Command Line Flags**: (e.g. `--connect 192.168.1.5:9090`)
     - **Terminal Log Output**: (Copy-paste log lines)
     - **Expected vs Actual Behavior**: (Screen freeze, input drop, etc.)
