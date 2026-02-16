#!/bin/bash

# Whois One-Click Packaging Script for macOS
set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🚀 Starting Whois app packaging process...${NC}"

# 1. Check for Node.js
if ! command -v node &> /dev/null; then
    echo -e "${RED}❌ Node.js is not installed. Please install it from https://nodejs.org/${NC}"
    exit 1
fi

# 2. Check for Rust
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Rust/Cargo is not installed. Please run:${NC}"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# 3. Ensure tauri-cli is installed
if ! command -v cargo-tauri &> /dev/null; then
    echo -e "${BLUE}📦 Installing Tauri CLI...${NC}"
    cargo install tauri-cli
fi

# 4. Install npm dependencies
echo -e "${BLUE}npm 🛠️ Installing frontend dependencies...${NC}"
npm install

# 5. Generate Icons
if [ -f "icon.svg" ]; then
    echo -e "${BLUE}🖼️ Generating macOS icons from icon.svg...${NC}"
    cargo tauri icon icon.svg
else
    echo -e "${RED}⚠️ icon.svg not found, skipping icon generation.${NC}"
fi

# 6. Build the application
echo -e "${BLUE}🏗️ Building Whois.app (this may take a few minutes)...${NC}"
# Note: This will build both the frontend and the Rust backend
cargo tauri build

echo -e "${GREEN}✅ Success! Whois has been packaged.${NC}"
echo -e "${GREEN}📂 Application location: ${NC}$(pwd)/src-tauri/target/release/bundle/macos/Whois.app"
echo -e "${BLUE}💡 You can find the DMG installer in the same directory.${NC}"
