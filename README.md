# Whois - Local Network Service Scanner

[中文](README_CN.md) | English

A high-performance local network service discovery tool built with Rust backend and modern React UI.

## 📦 Installation

### macOS
Download the latest `.dmg` file from [Releases](https://github.com/yorha59/whois/releases).

⚠️ **First Launch**: macOS will block unsigned applications. See [MACOS_INSTALL.md](MACOS_INSTALL.md) for solutions.

**Quick Fix**:
```bash
xattr -cr ~/Downloads/Whois_*.dmg
```

### Windows
Download and run the `.msi` or `.exe` installer from [Releases](https://github.com/yorha59/whois/releases).

## 🚀 Quick Release to GitHub

This project has GitHub Actions configured for automated builds. Simply push a tag to automatically build and publish macOS and Windows versions.

### Release Steps:

1. **Update Version**:
   Ensure `version` fields in `package.json` and `src-tauri/tauri.conf.json` match (e.g., `1.0.2`).

2. **Tag and Push**:
   ```bash
   git tag v1.0.2
   git push origin v1.0.2
   ```

3. **Monitor Progress**:
   Go to the **Actions** tab in your GitHub repository to see the "Publish Release" workflow running.

4. **Download Artifacts**:
   After completion, go to the **Releases** page. GitHub Actions will automatically create a draft release with all build artifacts:
   - macOS: `.dmg`, `.app.tar.gz`
   - Windows: `.msi`, `.exe`

## 🛠️ Local Development

See `README_MACOS.md` or `README_WINDOWS.md` for local environment setup and building.

### Quick Start Development

```bash
npm install
npm run tauri dev
```

## 🔐 Privacy & Security

This application uses Gemini AI for security analysis. Provide API Key via `API_KEY` environment variable (configured in `.env.local`).

## 📋 Features

- 🔍 Fast scanning of active hosts in local network
- 🔌 Identifies common service ports (SSH, HTTP, HTTPS, MySQL, PostgreSQL, Redis, etc.)
- 🤖 AI-powered security risk analysis
- 🎨 Modern dark theme interface
- ⚡ High-performance Rust-powered scanning engine

## 📝 Version Notes

- **v1.0.1**: Runtime fixes (Recommended)
  - Restored index.html entry script
  - Fixed Gemini SDK integration
  - Removed incompatible Tauri v2 configuration
  
- **v1.0.0**: Initial release (Has runtime issues, not recommended)

## 🤝 Contributing

Issues and Pull Requests are welcome!

## 📄 License

Copyright © 2024
