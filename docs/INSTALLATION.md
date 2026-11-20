# AgentDesk Installation Guide

> **Free & Open Source Remote Desktop — Made in Nepal 🇳🇵**  
> Company: [Next Gen Innovations Private Limited](https://nextgeninnovations.com.np/)  
> Developer: Abhishek Kumar Paswan | License: MIT

---

## 🪟 Windows

### Option A — Installation Wizard (Recommended)

1. Download **`AgentDesk-Setup-0.1.0.exe`** from [GitHub Releases](https://github.com/Lone-Wolf1X/AgentDesk/releases)
2. Double-click the installer — the **Setup Wizard** opens
3. Walk through the classic steps:
   - **Welcome** → Click *Next*
   - **License Agreement** → Read & Accept, click *Next*
   - **Select Destination Location** → Default is `C:\Program Files\AgentDesk` (or change to any folder), click *Next*
   - **Select Start Menu Folder** → Keep default `AgentDesk`, click *Next*
   - **Additional Tasks** → Tick "Create a Desktop shortcut" if desired, click *Next*
   - **Ready to Install** → Review summary, click *Install*
   - **Installing** → Progress bar runs (~5–10 seconds)
   - **Completing Setup** → Optionally tick "Launch AgentDesk now", click *Finish*
4. AgentDesk is now accessible from:
   - **Start Menu** → `AgentDesk`
   - **Desktop** → `AgentDesk` shortcut (if selected)

### Per-User Install (No Admin Required)

The installer will offer a dialog to install **per-user** (into `%LOCALAPPDATA%\Programs\AgentDesk`) if you don't have administrator rights. No UAC prompt needed.

### Uninstall (Windows)

- **Settings** → Apps → AgentDesk → Uninstall
- Or: **Control Panel** → Programs & Features → AgentDesk → Uninstall
- Or: Run `C:\Program Files\AgentDesk\unins000.exe` directly

---

## 🍏 macOS

### Download & Install

1. Download **`AgentDesk-0.1.0.dmg`** from [GitHub Releases](https://github.com/Lone-Wolf1X/AgentDesk/releases)
2. Double-click the `.dmg` file — a Finder window opens
3. Drag **AgentDesk.app** into the **Applications** folder (shortcut shown in the window)
4. Eject the disk image (drag to Trash or Eject button)
5. Open **Applications** → double-click **AgentDesk**

### First Launch (Gatekeeper Warning)

If you see *"AgentDesk cannot be opened because it is from an unidentified developer"*:

```
Right-click (or Ctrl+click) AgentDesk.app → Open → Open
```

This is a one-time step. After this, AgentDesk launches normally.

### Grant Required Permissions

macOS will prompt you the first time you use each feature:

| Permission | When Prompted | How to Grant |
|------------|--------------|--------------|
| **Screen Recording** | When someone connects to your Mac | System Preferences → Security & Privacy → Screen Recording → tick AgentDesk |
| **Accessibility** | When you control a remote machine | System Preferences → Security & Privacy → Accessibility → tick AgentDesk |

### Uninstall (macOS)

- Open **Finder** → **Applications** → right-click **AgentDesk** → **Move to Trash**
- Empty Trash

---

## 🐧 Linux

### Option A — .deb Package (Ubuntu / Debian / Linux Mint)

```bash
# Download the .deb file, then:
sudo dpkg -i agentdesk_0.1.0_amd64.deb
sudo apt-get install -f    # Fix any missing dependencies
```

Or simply **double-click** the `.deb` file to install via Ubuntu Software Center.

After install, launch from:
- **Applications Menu** → Internet → AgentDesk
- Terminal: `agentdesk`

**Uninstall:**
```bash
sudo apt remove agentdesk
```

---

### Option B — AppImage (Universal — Any Distro, No Install)

```bash
# Download AgentDesk-0.1.0-x86_64.AppImage, then:
chmod +x AgentDesk-0.1.0-x86_64.AppImage
./AgentDesk-0.1.0-x86_64.AppImage
```

No installation required. The AppImage is self-contained and runs on any modern Linux distro (Ubuntu 18.04+, Fedora 32+, Arch, etc.).

**Integrate into system (optional):**
```bash
# Move to ~/Applications for easy access
mkdir -p ~/Applications
mv AgentDesk-0.1.0-x86_64.AppImage ~/Applications/

# Create a desktop entry
./AgentDesk-0.1.0-x86_64.AppImage --appimage-extract-and-run
```

**Uninstall:** Simply delete the `.AppImage` file.

---

### Linux System Requirements

| Component | Requirement |
|-----------|-------------|
| Display Server | X11 (Wayland via XWayland also works) |
| Arch | x86_64 (amd64) |
| OS | Ubuntu 18.04+, Debian 10+, Fedora 32+, Arch, etc. |
| Libraries | `libx11-6`, `libxcb1`, `libxkbcommon0` (usually pre-installed) |

If libraries are missing (rare):
```bash
# Ubuntu/Debian
sudo apt install libx11-6 libxcb1 libxkbcommon0

# Fedora/RHEL
sudo dnf install libX11 libxcb libxkbcommon

# Arch
sudo pacman -S libx11 libxcb libxkbcommon
```

---

## 🚀 First-Time Use (All Platforms)

Once AgentDesk is running:

1. **"This Device" panel** (left side):
   - Your **IP address** and **Connection ID** are shown automatically
   - Share these with the person who wants to connect to you
   - When they connect, an **Accept/Reject popup** appears — you decide

2. **"Connect to Remote" panel** (right side):
   - Enter the remote machine's **IP address** (e.g. `192.168.1.15`) or **Connection ID**
   - Click **Connect** — the remote host must Accept your request
   - Once accepted, the remote screen appears and you can control it

> 💡 **Phase 1 (LAN only)**: Both machines must be on the **same WiFi or wired network**.  
> Phase 2 will add internet relay for cross-network connections.

---

## 🏗️ Build from Source

If you prefer to build AgentDesk yourself:

```bash
# Prerequisites: Rust stable toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/Lone-Wolf1X/AgentDesk.git
cd AgentDesk
cargo build --release -p agentdesk

# Binary locations:
# Windows:  target\release\agentdesk.exe
# macOS:    target/release/agentdesk
# Linux:    target/release/agentdesk
```

See [BUILD_DISTRIBUTION.md](BUILD_DISTRIBUTION.md) for creating native installers from source.

---

## 📄 License & Credits

MIT License — Copyright (c) 2025 **Next Gen Innovations Private Limited** and **Abhishek Kumar Paswan**

- Website: [https://nextgeninnovations.com.np/](https://nextgeninnovations.com.np/)
- GitHub: [https://github.com/Lone-Wolf1X/AgentDesk](https://github.com/Lone-Wolf1X/AgentDesk)
- 100% Free — No paid tiers, no subscriptions, no limits, ever.
