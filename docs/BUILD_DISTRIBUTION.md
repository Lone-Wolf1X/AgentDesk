# 📦 AgentDesk Distribution & Packaging Guide (Single Unified Binary)

AgentDesk ab **ek single standalone executable (`agentdesk`)** produce karta hai jo Host aur Viewer dono roles support karta hai.

---

## A. Windows — `.exe` & Installer Build

### 1. Build Standalone Executable
```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --workspace
```
Output:
- `target\release\agentdesk.exe`
- `target\release\agentdesk-signal.exe`

Yeh single executable bina kisi external runtime ya dependency ke Windows 10/11 par directly double-click se run hota hai.

### 2. Inno Setup Setup Installer (`AgentDesk-Setup-0.1.0.exe`)
`packaging\windows\installer.iss` script use karein:
```cmd
"C:\Program Files (x86)\Inno Setup 6\ISCC.exe" packaging\windows\installer.iss
```
Output:
`packaging\windows\Output\AgentDesk-Setup-0.1.0.exe` (Single install wizard jo Desktop aur Start Menu shortcuts create karta hai).

---

## B. macOS — Universal Binary & `.dmg` Build

### 1. Universal Binary (Intel + Apple Silicon)
```bash
rustup target add aarch64-apple-darwin x86_64-apple-darwin

cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin

mkdir -p target/release/universal
lipo -create \
  target/aarch64-apple-darwin/release/agentdesk \
  target/x86_64-apple-darwin/release/agentdesk \
  -output target/release/universal/agentdesk
```

### 2. Create `.app` Bundle
```bash
mkdir -p AgentDesk.app/Contents/MacOS
mkdir -p AgentDesk.app/Contents/Resources
cp target/release/universal/agentdesk AgentDesk.app/Contents/MacOS/agentdesk
cp packaging/macos/Info.plist AgentDesk.app/Contents/Info.plist
chmod +x AgentDesk.app/Contents/MacOS/*
```

### 3. Generate `.dmg`
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

---

## C. Linux — `.AppImage` & `.deb` Build

### 1. AppImage
```bash
wget https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
chmod +x linuxdeploy-x86_64.AppImage

cargo build --release --workspace

mkdir -p AppDir/usr/bin
cp target/release/agentdesk AppDir/usr/bin/
cp packaging/linux/agentdesk.desktop AppDir/
cp packaging/linux/agentdesk.png AppDir/ 2>/dev/null || touch AppDir/agentdesk.png

./linuxdeploy-x86_64.AppImage --appdir AppDir --output appimage
```

### 2. Debian / Ubuntu Package (`.deb`)
```bash
cargo install cargo-deb
cargo deb --package agentdesk
```

---

## D. Automated GitHub Releases

Tag push par GitHub Actions automatically sabhi packages build aur publish karta hai:
```bash
git tag v0.1.0
git push origin v0.1.0
```
Outputs in GitHub Releases:
- `agentdesk-windows-x86_64.zip`
- `agentdesk-macos-arm64.tar.gz`
- `agentdesk-macos-x86_64.tar.gz`
- `agentdesk-linux-x86_64.tar.gz`
