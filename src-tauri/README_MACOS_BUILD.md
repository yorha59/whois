# macOS Network Scanning Entitlements

## Problem

On macOS, CLI applications may fail with "No route to host" (errno=65) when attempting TCP connections, even though the network is reachable.

This happens because macOS requires specific **entitlements** for network operations.

## Solution

### Build with Entitlements

Use the provided build script:

```bash
cd src-tauri
chmod +x build_and_sign.sh
./build_and_sign.sh
```

Or manually sign after building:

```bash
cargo build --release
codesign --force --sign - --entitlements entitlements.plist target/release/rust-net-scanner-backend
```

### Entitlements

The `entitlements.plist` file grants:

- `com.apple.security.network.client` - Outgoing connections
- `com.apple.security.network.server` - Incoming connections

These are the same entitlements that Apple's `/usr/bin/nc` has.

## Verification

```bash
./target/release/rust-net-scanner-backend --scan
```

## Why This Happens

- `/usr/bin/nc` is pre-signed by Apple with network entitlements
- Unsigned/ad-hoc signed binaries lack these permissions
- The kernel blocks certain network operations without proper entitlements

## GUI Alternative

The Tauri GUI application has network permissions by default:

```bash
npm run tauri dev
```
