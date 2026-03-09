#!/bin/bash
set -e

# Build macOS universal binary and create DMG
# This script works around Tauri 1.x DMG bundling issues

echo "🚀 Building macOS universal binary..."

# Read version from package.json
VERSION=$(node -p "require('./package.json').version")
APP_NAME="OpenClawSwitch"

# Build .app bundle only (skip DMG to avoid Tauri 1.x bug)
npm run tauri build -- --target universal-apple-darwin --bundles app

# Paths
APP_PATH="src-tauri/target/universal-apple-darwin/release/bundle/macos/${APP_NAME}.app"
DMG_PATH="src-tauri/target/universal-apple-darwin/release/bundle/${APP_NAME}_${VERSION}_universal.dmg"

# Verify .app exists
if [ ! -d "$APP_PATH" ]; then
  echo "❌ Error: .app bundle not found at $APP_PATH"
  exit 1
fi

echo "✅ .app bundle created successfully"

# Create DMG using hdiutil
echo "📦 Creating DMG..."
hdiutil create \
  -volname "$APP_NAME" \
  -srcfolder "$APP_PATH" \
  -ov \
  -format UDZO \
  "$DMG_PATH"

if [ -f "$DMG_PATH" ]; then
  DMG_SIZE=$(du -h "$DMG_PATH" | cut -f1)
  echo "✅ DMG created successfully: $DMG_PATH ($DMG_SIZE)"
else
  echo "❌ Error: DMG creation failed"
  exit 1
fi

echo "🎉 Build complete!"
