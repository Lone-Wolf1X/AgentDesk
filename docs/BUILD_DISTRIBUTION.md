# 📦 AgentDesk Distribution & Packaging Guide

Yeh guide explain karti hai ki kaise **AgentDesk** ke standalone release binaries aur installers compile kiye jate hain har platform ke liye:
- **Windows**: `.exe` standalone & Inno Setup `.exe`/`.msi` installer
- **macOS**: Universal Binary (`x86_64` + `aarch64`) aur `.dmg` drag-and-drop installer
- **Linux**: Universal `.AppImage`, Ubuntu/Debian `.deb`, aur Fedora `.rpm` packages

---

## A. Windows — `.exe` & Installer Build

### 1. Standalone Release Build
Windows par release executable build karne ke liye:

```powershell
# MSVC Target set karein
rustup target add x86_64-pc-windows-msvc

# Build workspace in release mode
cargo build --release --workspace
```

Output executables:
- `target\release\agentdesk-host.exe`
- `target\release\agentdesk-viewer.exe`
- `target\release\agentdesk-signal.exe`

Yeh files self-contained hoti hain. Aap directly ek `.zip` folder mein pack karke end-users ko share kar sakte hain — users ko Rust install karne ki zaroorat nahi hai.

### 2. Inno Setup Installer (`AgentDesk-Setup-0.1.0.exe`)
Professional installer banane ke liye [Inno Setup](https://jrsoftware.org/isdl.php) (free & open source) use karein:

1. Inno Setup download aur install karein.
2. Inno Setup Compiler kholein aur repo ki packaging script load karein:
   `packaging\windows\installer.iss`
3. Click **Compile** (ya command line se run karein):
   ```cmd
   "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" packaging\windows\installer.iss
   ```
4. Output installer create ho jayega:
   `packaging\windows\Output\AgentDesk-Setup-0.1.0.exe`

### 3. Code Signing (Optional)
Agar aapke paas digital certificate hai toh Windows SmartScreen warning hatane ke liye sign karein:
```powershell
signtool sign /tr http://timestamp.digicert.com /td sha256 /fd sha256 /a "packaging\windows\Output\AgentDesk-Setup-0.1.0.exe"
```

---

## B. macOS — Universal Binary & `.dmg` Build

### 1. Universal Binary (Intel + Apple Silicon M1/M2/M3/M4)
Dono Mac architectures support karne ke liye:

```bash
# Dono target toolchains install karein
rustup target add aarch64-apple-darwin x86_64-apple-darwin

# Apple Silicon ke liye build karein
cargo build --release --target aarch64-apple-darwin

# Intel Mac ke liye build karein
cargo build --release --target x86_64-apple-darwin

# Lipotool se Universal Binaries create karein
mkdir -p target/release/universal
lipo -create \
  target/aarch64-apple-darwin/release/agentdesk-host \
  target/x86_64-apple-darwin/release/agentdesk-host \
  -output target/release/universal/agentdesk-host

lipo -create \
  target/aarch64-apple-darwin/release/agentdesk-viewer \
  target/x86_64-apple-darwin/release/agentdesk-viewer \
  -output target/release/universal/agentdesk-viewer
```

### 2. Create `.app` Bundle
macOS application bundle structure:

```
AgentDesk.app/
├── Contents/
│   ├── Info.plist
│   ├── MacOS/
│   │   ├── agentdesk-host
│   │   └── agentdesk-viewer
│   └── Resources/
│       └── icon.icns
```

Commands:
```bash
mkdir -p AgentDesk.app/Contents/MacOS
mkdir -p AgentDesk.app/Contents/Resources

cp target/release/universal/agentdesk-viewer AgentDesk.app/Contents/MacOS/agentdesk-viewer
cp target/release/universal/agentdesk-host AgentDesk.app/Contents/MacOS/agentdesk-host
cp packaging/macos/Info.plist AgentDesk.app/Contents/Info.plist
chmod +x AgentDesk.app/Contents/MacOS/*
```

### 3. Create `.dmg` Installer
[create-dmg](https://github.com/create-dmg/create-dmg) tool use karein (`brew install create-dmg`):

```bash
create-dmg \
  --volname "AgentDesk" \
  --window-pos 200 120 \
  --window-size 600 400 \
  --icon-size 100 \
  --icon "AgentDesk.app" 175 190 \
  --hide-extension "AgentDesk.app" \
  --app-drop-link 425 190 \
  "AgentDesk-0.1.0.dmg" \
  "AgentDesk.app"
```

### 4. Code Signing & Notarization (macOS Gatekeeper)
```bash
# Code sign with Apple Developer ID
codesign --deep --force --verify --verbose \
  --sign "Developer ID Application: Next Gen Innovations Private Limited" \
  AgentDesk.app

# Notarize DMG with Apple
xcrun notarytool submit AgentDesk-0.1.0.dmg \
  --apple-id "developer@nextgeninnovations.com.np" \
  --team-id "TEAMID" \
  --password "app-specific-password" \
  --wait
```

---

## C. Linux — `.AppImage`, `.deb`, & `.rpm` Build

### 1. AppImage (Universal Linux Package)
AppImage sabhi major distros (Ubuntu, Fedora, Arch, Debian, openSUSE) par chalta hai:

```bash
# 1. Download linuxdeploy
wget https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
chmod +x linuxdeploy-x86_64.AppImage

# 2. Build binaries
cargo build --release --workspace

# 3. Setup AppDir
mkdir -p AppDir/usr/bin
cp target/release/agentdesk-host AppDir/usr/bin/
cp target/release/agentdesk-viewer AppDir/usr/bin/
cp packaging/linux/agentdesk.desktop AppDir/
cp packaging/linux/agentdesk.png AppDir/agentdesk.png 2>/dev/null || touch AppDir/agentdesk.png

# 4. Generate AppImage
./linuxdeploy-x86_64.AppImage --appdir AppDir --output appimage
# Result: AgentDesk-0.1.0-x86_64.AppImage
```

### 2. Debian / Ubuntu Package (`.deb`)
```bash
# Install cargo-deb tool
cargo install cargo-deb

# Build debian packages
cargo deb --package agentdesk-host
cargo deb --package agentdesk-viewer

# Output:
# target/debian/agentdesk-host_0.1.0_amd64.deb
# target/debian/agentdesk-viewer_0.1.0_amd64.deb
```

Install command on user PC:
```bash
sudo dpkg -i target/debian/agentdesk-viewer_0.1.0_amd64.deb
```

### 3. RedHat / Fedora Package (`.rpm`)
```bash
cargo install cargo-rpm
cargo rpm build
```

---

## D. Automated Multi-Platform Releases (GitHub Actions)

Humne `.github/workflows/release.yml` automate kar diya hai. 

Jab bhi aap git par release version tag push karenge:
```bash
git tag v0.1.0
git push origin v0.1.0
```

GitHub Actions automatically:
1. Windows MSVC par `.exe` binaries build karega.
2. macOS Intel aur Apple Silicon par Universal Binaries compile karega.
3. Linux Ubuntu par binaries aur packages generate karega.
4. Sabhi artifacts ko **GitHub Releases** page par auto-publish kar dega!
