#!/bin/bash
# AgentDesk macOS .dmg Builder
# Copyright (c) 2025 Next Gen Innovations Private Limited
# Developer: Abhishek Kumar Paswan
# Made in Nepal 🇳🇵
#
# Prerequisites:
#   rustup target add aarch64-apple-darwin x86_64-apple-darwin
#   brew install create-dmg
#
# Usage:
#   chmod +x packaging/macos/build-dmg.sh
#   bash packaging/macos/build-dmg.sh
#
# Optional env vars:
#   APPLE_SIGNING_IDENTITY  — e.g. "Developer ID Application: Next Gen Innovations (TEAMID)"
#   APPLE_ID, APPLE_TEAM_ID, APPLE_APP_PASSWORD — for notarization

set -euo pipefail

APP_NAME="AgentDesk"
VERSION="0.1.0"
BUNDLE_ID="com.nextgeninnovations.agentdesk"
BUNDLE="packaging/macos/${APP_NAME}.app"
DMG_OUT="dist/macos/${APP_NAME}-${VERSION}.dmg"
MACOS_BIN="${BUNDLE}/Contents/MacOS/agentdesk"
RESOURCES="${BUNDLE}/Contents/Resources"

echo "==================================================="
echo "  AgentDesk macOS Build Script"
echo "  Version: ${VERSION}"
echo "  Next Gen Innovations Private Limited"
echo "  Made in Nepal 🇳🇵"
echo "==================================================="

# 1. Build universal binary (Apple Silicon + Intel)
echo ""
echo "→ Building for aarch64-apple-darwin (Apple Silicon)..."
cargo build --release --target aarch64-apple-darwin -p agentdesk

echo "→ Building for x86_64-apple-darwin (Intel)..."
cargo build --release --target x86_64-apple-darwin -p agentdesk

echo "→ Creating Universal Binary with lipo..."
mkdir -p "${BUNDLE}/Contents/MacOS"
lipo -create \
    "target/aarch64-apple-darwin/release/agentdesk" \
    "target/x86_64-apple-darwin/release/agentdesk" \
    -output "${MACOS_BIN}"
chmod +x "${MACOS_BIN}"

echo "✅ Universal binary created: ${MACOS_BIN}"

# 2. Copy Resources
echo "→ Setting up app bundle resources..."
mkdir -p "${RESOURCES}"

# Copy LICENSE
if [ -f "LICENSE" ]; then
    cp "LICENSE" "${RESOURCES}/LICENSE.txt"
fi

# Convert PNG to ICNS (requires iconutil or sips)
if [ -f "assets/icon.png" ]; then
    ICONSET_DIR="${RESOURCES}/icon.iconset"
    mkdir -p "${ICONSET_DIR}"

    # Generate all required sizes
    for SIZE in 16 32 64 128 256 512; do
        sips -z ${SIZE} ${SIZE} "assets/icon.png" --out "${ICONSET_DIR}/icon_${SIZE}x${SIZE}.png" 2>/dev/null
        DOUBLE=$((SIZE * 2))
        sips -z ${DOUBLE} ${DOUBLE} "assets/icon.png" --out "${ICONSET_DIR}/icon_${SIZE}x${SIZE}@2x.png" 2>/dev/null
    done

    iconutil -c icns "${ICONSET_DIR}" -o "${RESOURCES}/icon.icns"
    rm -rf "${ICONSET_DIR}"
    echo "✅ icon.icns created"
fi

# 3. Code sign (optional — requires Apple Developer ID)
if [ -n "${APPLE_SIGNING_IDENTITY:-}" ]; then
    echo "→ Code signing with identity: ${APPLE_SIGNING_IDENTITY}"
    codesign --deep --force --options runtime \
        --entitlements "packaging/macos/entitlements.plist" \
        --sign "${APPLE_SIGNING_IDENTITY}" \
        "${BUNDLE}"
    echo "✅ Code signing complete"
else
    echo "⚠️  Skipping code signing (APPLE_SIGNING_IDENTITY not set)"
    echo "   First-time users will need to: right-click → Open to bypass Gatekeeper"
fi

# 4. Build .dmg with drag-to-Applications UX
echo ""
echo "→ Building .dmg..."
mkdir -p dist/macos

# Remove old .dmg if exists
[ -f "${DMG_OUT}" ] && rm "${DMG_OUT}"

create-dmg \
    --volname "${APP_NAME} ${VERSION}" \
    --volicon "${RESOURCES}/icon.icns" \
    --window-pos 200 120 \
    --window-size 600 420 \
    --icon-size 100 \
    --icon "${APP_NAME}.app" 175 200 \
    --hide-extension "${APP_NAME}.app" \
    --app-drop-link 425 200 \
    --hdiutil-quiet \
    "${DMG_OUT}" \
    "packaging/macos/"

echo ""
echo "==================================================="
echo "✅ DMG created: ${DMG_OUT}"
echo "==================================================="

# 5. Notarize (optional — requires Apple Developer account)
if [ -n "${APPLE_ID:-}" ]; then
    echo ""
    echo "→ Submitting to Apple Notarization service..."
    xcrun notarytool submit "${DMG_OUT}" \
        --apple-id "${APPLE_ID}" \
        --team-id "${APPLE_TEAM_ID}" \
        --password "${APPLE_APP_PASSWORD}" \
        --wait

    echo "→ Stapling notarization ticket..."
    xcrun stapler staple "${DMG_OUT}"
    echo "✅ Notarization complete — ready for distribution"
else
    echo "⚠️  Skipping notarization (APPLE_ID not set)"
fi
