#!/bin/bash
# AgentDesk Linux AppImage Builder
# Copyright (c) 2025 Next Gen Innovations Private Limited
# Developer: Abhishek Kumar Paswan
# Made in Nepal 🇳🇵
#
# Prerequisites (Ubuntu/Debian):
#   sudo apt-get install libx11-dev libxcb1-dev libxkbcommon-dev
#   # wget is used to download linuxdeploy
#
# Usage:
#   chmod +x packaging/linux/build-appimage.sh
#   bash packaging/linux/build-appimage.sh

set -euo pipefail

APP_NAME="agentdesk"
DISPLAY_NAME="AgentDesk"
VERSION="0.1.0"
APPDIR="build/AppDir"
OUTPUT_DIR="dist/linux"
APPIMAGE_OUT="${OUTPUT_DIR}/${DISPLAY_NAME}-${VERSION}-x86_64.AppImage"

echo "==================================================="
echo "  AgentDesk Linux AppImage Builder"
echo "  Version: ${VERSION}"
echo "  Next Gen Innovations Private Limited"
echo "  Made in Nepal"
echo "==================================================="

# 1. Build release binary
echo ""
echo "→ Building release binary..."
cargo build --release -p agentdesk
echo "✅ Binary built: target/release/agentdesk"

# 2. Create AppDir structure (FHS layout inside AppDir)
echo "→ Assembling AppDir..."
rm -rf "${APPDIR}"
mkdir -p "${APPDIR}/usr/bin"
mkdir -p "${APPDIR}/usr/share/applications"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/256x256/apps"
mkdir -p "${APPDIR}/usr/share/doc/agentdesk"
mkdir -p "${APPDIR}/usr/share/metainfo"

# Copy binary
cp "target/release/agentdesk" "${APPDIR}/usr/bin/${APP_NAME}"
chmod +x "${APPDIR}/usr/bin/${APP_NAME}"

# Copy desktop entry
cp "packaging/linux/agentdesk.desktop" "${APPDIR}/usr/share/applications/"

# Copy icon
cp "packaging/linux/agentdesk.png" "${APPDIR}/usr/share/icons/hicolor/256x256/apps/"

# Copy docs
[ -f "README.md" ] && cp "README.md" "${APPDIR}/usr/share/doc/agentdesk/"
[ -f "LICENSE" ]   && cp "LICENSE"   "${APPDIR}/usr/share/doc/agentdesk/copyright"

# AppStream metadata (optional but recommended for software centers)
cat > "${APPDIR}/usr/share/metainfo/com.nextgeninnovations.agentdesk.appdata.xml" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
  <id>com.nextgeninnovations.agentdesk</id>
  <name>AgentDesk</name>
  <summary>Free &amp; Open Source Remote Desktop — Made in Nepal</summary>
  <description>
    <p>AgentDesk is a high-performance, 100% free and open source remote desktop application.
       Connect to any device, control keyboard and mouse, and share screens with zero limits.</p>
    <p>No time limits, no device limits, no account required, no telemetry.</p>
  </description>
  <url type="homepage">https://nextgeninnovations.com.np/</url>
  <developer_name>Next Gen Innovations Private Limited</developer_name>
  <metadata_license>MIT</metadata_license>
  <project_license>MIT</project_license>
  <categories>
    <category>Network</category>
    <category>RemoteAccess</category>
  </categories>
  <releases>
    <release version="0.1.0" date="2025-01-01"/>
  </releases>
</component>
EOF

# 3. AppImage root-level symlinks (required by AppImage spec)
ln -sf "usr/share/applications/agentdesk.desktop" "${APPDIR}/agentdesk.desktop"
ln -sf "usr/share/icons/hicolor/256x256/apps/agentdesk.png" "${APPDIR}/agentdesk.png"
ln -sf "usr/share/icons/hicolor/256x256/apps/agentdesk.png" "${APPDIR}/.DirIcon"

echo "✅ AppDir assembled: ${APPDIR}"

# 4. Download linuxdeploy if not present
mkdir -p "${OUTPUT_DIR}"
LINUXDEPLOY="./linuxdeploy-x86_64.AppImage"

if [ ! -f "${LINUXDEPLOY}" ]; then
    echo "→ Downloading linuxdeploy..."
    wget -q --show-progress \
        "https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage" \
        -O "${LINUXDEPLOY}"
    chmod +x "${LINUXDEPLOY}"
fi

# 5. Build AppImage
echo "→ Building AppImage..."
OUTPUT="${APPIMAGE_OUT}" \
ARCH=x86_64 \
    "${LINUXDEPLOY}" \
    --appdir "${APPDIR}" \
    --output appimage

# linuxdeploy outputs to current dir, move to dist/
if [ -f "AgentDesk-${VERSION}-x86_64.AppImage" ]; then
    mv "AgentDesk-${VERSION}-x86_64.AppImage" "${APPIMAGE_OUT}"
elif ls AgentDesk*.AppImage 2>/dev/null | head -1 | grep -q .; then
    mv "$(ls AgentDesk*.AppImage | head -1)" "${APPIMAGE_OUT}"
fi

chmod +x "${APPIMAGE_OUT}"

echo ""
echo "==================================================="
echo "✅ AppImage created: ${APPIMAGE_OUT}"
echo ""
echo "Usage:"
echo "  chmod +x ${APPIMAGE_OUT}"
echo "  ./${APPIMAGE_OUT}"
echo "==================================================="
