#!/bin/bash
# Build Whois for macOS (supports headless environments)
# Usage: ./build_macos.sh

set -e

echo "🔧 Building Whois for macOS..."

# Build frontend
echo "📦 Building frontend..."
npm run build

# Build Tauri (will fail at DMG step on headless, that's OK)
echo "🦀 Building Tauri backend..."
npx tauri build 2>&1 || true

# Check if .app was created
APP_PATH="src-tauri/target/release/bundle/macos/Whois.app"
if [ ! -d "$APP_PATH" ]; then
    echo "❌ Whois.app not found! Build failed."
    exit 1
fi
echo "✅ Whois.app built successfully"

# Create DMG manually with sandbox-safe mode (works on headless Mac)
DMG_DIR="src-tauri/target/release/bundle/dmg"
DMG_PATH="$DMG_DIR/Whois_$(grep '"version"' src-tauri/tauri.conf.json | head -1 | sed 's/[^0-9.]//g')_$(uname -m).dmg"

if [ -f "$DMG_DIR/bundle_dmg.sh" ]; then
    echo "📀 Creating DMG (sandbox-safe mode for headless compatibility)..."
    cd "$DMG_DIR"
    bash bundle_dmg.sh --sandbox-safe \
        --volname "Whois" \
        --icon "Whois.app" 180 170 \
        --app-drop-link 480 170 \
        --window-size 660 400 \
        --hide-extension "Whois.app" \
        "$(basename $DMG_PATH)" \
        "$(cd ../../macos && pwd)/Whois.app"
    cd -
    echo "✅ DMG created: $DMG_PATH"
    ls -lh "$DMG_PATH"
else
    echo "⚠️  bundle_dmg.sh not found, skipping DMG creation"
fi

echo ""
echo "🎉 Build complete!"
echo "   App: $APP_PATH"
[ -f "$DMG_PATH" ] && echo "   DMG: $DMG_PATH"
