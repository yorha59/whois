# Whois One-Click Packaging Script for Windows
$ErrorActionPreference = "Stop"

Write-Host "🚀 Starting Whois app packaging process for Windows..." -ForegroundColor Cyan

# 1. Check for Node.js
if (!(Get-Command node -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Node.js is not installed. Please install it from https://nodejs.org/" -ForegroundColor Red
    exit
}

# 2. Check for Rust
if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Rust/Cargo is not installed. Please install it from https://rustup.rs/" -ForegroundColor Red
    exit
}

# 3. Ensure Tauri CLI is installed
if (!(Get-Command cargo-tauri -ErrorAction SilentlyContinue)) {
    Write-Host "📦 Installing Tauri CLI..." -ForegroundColor Blue
    cargo install tauri-cli
}

# 4. Install npm dependencies
Write-Host "npm 🛠️ Installing frontend dependencies..." -ForegroundColor Blue
npm install

# 5. Generate Icons (if icon.svg exists)
if (Test-Path "icon.svg") {
    Write-Host "🖼️ Generating Windows icons from icon.svg..." -ForegroundColor Blue
    cargo tauri icon icon.svg
} else {
    Write-Host "⚠️ icon.svg not found, skipping icon generation." -ForegroundColor Yellow
}

# 6. Build the application
Write-Host "🏗️ Building Whois.exe (this may take a few minutes)..." -ForegroundColor Blue
# This triggers the full tauri build pipeline
cargo tauri build

$outputPath = Join-Path (Get-Location) "src-tauri\target\release\bundle"
Write-Host "`n✅ Success! Whois has been packaged." -ForegroundColor Green
Write-Host "📂 Application location: $outputPath" -ForegroundColor Green
Write-Host "💡 You can find the .msi installer and the standalone .exe in the subfolders." -ForegroundColor Blue
