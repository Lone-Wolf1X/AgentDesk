# 🧪 AgentDesk Testing Guide: Do Real PCs Par Testing Kaise Karein (Single-App Architecture)

Yeh guide aapko step-by-step batati hai ki kaise do physical machines (PCs / Laptops) par **AgentDesk** ko run aur test karna hai.

AgentDesk AnyDesk aur TeamViewer ki tarah **ek hi single app** hai jo Host aur Viewer dono roles perform karta hai. Dono PCs par alag-alag binaries download karne ki zaroorat nahi hai — **dono taraf bas `agentdesk` run karein!**

---

## 📌 Architecture: Single App, Dual Role

```
+-------------------------------------+                         +-------------------------------------+
|        PC-1 (Target Machine)        |                         |       PC-2 (Client Machine)         |
|                                     |                         |                                     |
|  Runs: agentdesk                    | <=== H.264 Video Stream |  Runs: agentdesk                    |
|  Shows: "Your IP: 192.168.1.15:9090"| ===> Keyboard/Mouse === |  Enters: 192.168.1.15:9090          |
|  Password: "x7k2m9p4"               |      (Port 9090 / TCP)  |  Password: "x7k2m9p4"               |
|  Status: 🟢 Waiting for connection  |                         |  Clicks: [🚀 Connect]               |
+-------------------------------------+                         +-------------------------------------+
```

---

## A. Prerequisites (Dono PCs Ke Liye)

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
- **Permissions**:
  - `System Settings` -> `Privacy & Security` -> `Screen Recording` -> Terminal ya AgentDesk ko **Allow** karein.
  - `System Settings` -> `Privacy & Security` -> `Accessibility` -> Terminal ya AgentDesk ko **Allow** karein.

### 3. Linux Machine
- **Session**: X11 session zaroori hai (Wayland abhi experimental hai).
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
- Shuruat ke liye **dono PCs same WiFi router ya local LAN switch se connected hon**.

---

## B. Step 1 — Build on Both Machines

Dono machines par repository clone karein aur release binary compile karein:

```bash
git clone https://github.com/Lone-Wolf1X/AgentDesk.git
cd AgentDesk
cargo build --release --workspace
```

### Compiled Binary Ki Location:
- **Windows**: `target\release\agentdesk.exe`
- **macOS / Linux**: `target/release/agentdesk`

---

## C. Step 2 — Start App on PC-1 (Target / Host Machine)

Jis PC ko remote control karna hai, uspar double-click karke ya terminal se `agentdesk` run karein:

```powershell
# Windows PowerShell
.\target\release\agentdesk.exe
```
```bash
# macOS / Linux
./target/release/agentdesk
```

### Screen Par Kya Dikhega:
1. AgentDesk ki **Unified Dual-Panel Window** open hogi:
   - **🖥️ This Device Section**:
     - **Your IP Address**: e.g. `192.168.1.15:9090` (sath mein `[📋 Copy]` button)
     - **Your Connection ID**: e.g. `482-195-730`
     - **Access Password**: Auto-generated 8-character password e.g. `x7k2m9p4` (`[📋 Copy]` button ke sath)
     - Status: `🟢 Status: Waiting for connections`
2. **Firewall Prompt (First Launch)**:
   - Windows Defender Firewall ka prompt aayega -> **"Allow access"** click karein.
   - ![Firewall Prompt](images/firewall_prompt.png)

---

## D. Step 3 — Start App on PC-2 & Connect (Client Machine)

Ab controlling machine (PC-2) par bhi wahi same application run karein:

```bash
./target/release/agentdesk
```

### Connection Kaise Karein:
1. Window ke right side mein **"🔗 Connect to Remote"** section dekhein.
2. **Remote IP or 9-Digit ID**: PC-1 ka IP address daalein (e.g. `192.168.1.15:9090`).
3. **Remote Password**: PC-1 ka password daalein (e.g. `x7k2m9p4`).
4. **[🚀 Connect to Remote]** button par click karein.

*(Note: Advanced users CLI se direct connect bhi kar sakte hain:*
`./agentdesk connect --connect 192.168.1.15:9090 --password x7k2m9p4`*)*

---

## E. Step 4 — Accept Connection on PC-1

1. PC-1 par popup alert aayega:
   - `"⚠️ Incoming connection request from 192.168.1.42. Allow remote viewer to view and control your desktop?"`
2. **[✅ Accept Access]** par click karein.
3. PC-2 ki window instant live remote desktop view mein convert ho jayegi!

---

## F. Step 5 — Verification Checklist

Connect hone ke baad yeh checklist test karein:

| Test | Expected Result | Pass / Fail |
|---|---|---|
| **Remote Screen Visibility** | PC-2 window mein PC-1 ka poora desktop live dikhna chahiye. | `[  ]` |
| **Mouse Cursor Movement** | PC-2 mein mouse move karein -> PC-1 par cursor sync move ho. | `[  ]` |
| **Mouse Click & Double Click** | PC-2 se folder/file open karein -> PC-1 par open ho. | `[  ]` |
| **Keyboard Typing** | Notepad kholein aur type karein -> real-time typing ho. | `[  ]` |
| **Keyboard Shortcuts** | `Ctrl+C`, `Ctrl+V` check karein. | `[  ]` |
| **Stream Stability** | 10 minute tak continuous chalayein, koi lag ya drop na ho. | `[  ]` |
| **Session Disconnect** | Top bar par `[❌ Disconnect]` click karein -> dono PCs safely idle state mein aa jayein. | `[  ]` |

---

## G. Troubleshooting

- **"Connection timed out"**: PC-1 ka firewall port `9090` block kar raha hai. Windows firewall mein port 9090 TCP inbound allow karein:
  `New-NetFirewallRule -DisplayName "AgentDesk" -Direction Inbound -LocalPort 9090 -Protocol TCP -Action Allow`
- **Black Screen on macOS**: `System Settings` -> `Screen Recording` permission allow karein.
- **Mouse/Keyboard Input Not Working**: macOS par `Accessibility` permission allow karein; Windows par agar target window Admin privilege se chali hai toh AgentDesk ko bhi "Run as Administrator" chalayein.
- **Port 9090 already in use**: AgentDesk automatically agla available port bind karta hai (9091, 9092...) aur screen par exact bound port dikhata hai.

---

## H. Reporting Issues
Agar koi issue mile toh logs ke sath issue report karein:
- **GitHub Issues**: [AgentDesk Issues](https://github.com/Lone-Wolf1X/AgentDesk/issues)
- Run with debug logs:
  `$env:RUST_LOG="debug"; .\target\release\agentdesk.exe`
