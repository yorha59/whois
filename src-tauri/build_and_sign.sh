#!/bin/bash
# Build and sign script for macOS
# Adds network entitlements required for port scanning

set -e

echo "Building release binary..."
cargo build --release

BINARY="target/release/rust-net-scanner-backend"

if [ -f "$BINARY" ]; then
    echo "Signing binary with network entitlements..."
    codesign --force --sign - --entitlements entitlements.plist "$BINARY"
    echo "✅ Build and signing complete!"
else
    echo "❌ Binary not found at $BINARY"
    exit 1
fi
