# Whois - 局域网服务扫描器

一个基于 Rust 高性能后端和 React 现代化 UI 的局域网服务发现工具。

## 📦 安装

### macOS
从 [Releases](https://github.com/yorha59/whois/releases) 下载最新的 `.dmg` 文件。

⚠️ **首次运行**: macOS 会阻止未签名的应用。请查看 [MACOS_INSTALL.md](MACOS_INSTALL.md) 了解如何解决。

**快速解决**:
```bash
xattr -cr ~/Downloads/Whois_*.dmg
```

### Windows
从 [Releases](https://github.com/yorha59/whois/releases) 下载 `.msi` 或 `.exe` 安装包并运行。

## 🚀 快速发布到 GitHub Release

本项目已配置 GitHub Actions 自动化流水线。只需推送标签即可自动构建并发布 macOS 和 Windows 版本。

### 发布步骤：

1. **更新版本号**：
   确保 `package.json` 和 `src-tauri/tauri.conf.json` 中的 `version` 字段是一致的（例如 `1.0.2`）。

2. **打标签并推送**：
   在终端运行：
   ```bash
   git tag v1.0.2
   git push origin v1.0.2
   ```

3. **查看进度**：
   前往 GitHub 仓库的 **Actions** 选项卡，你可以看到 "Publish Release" 工作流正在运行。

4. **下载产物**：
   构建完成后，前往 **Releases** 页面。GitHub Action 会自动创建一个草稿（Draft）发布，并上传所有构建好的安装包：
   - macOS: `.dmg`, `.app.tar.gz`
   - Windows: `.msi`, `.exe`

## 🛠️ 本地开发

参照 `README_MACOS.md` 或 `README_WINDOWS.md` 进行本地环境配置和构建。

### 快速启动开发环境

```bash
npm install
npm run tauri dev
```

## 🔐 隐私与安全

本应用使用 Gemini AI 进行安全分析。API Key 请通过环境变量 `API_KEY` 提供（在 `.env.local` 中配置）。

## 📋 功能特性

- 🔍 快速扫描局域网内的活跃主机
- 🔌 识别常见服务端口 (SSH, HTTP, HTTPS, MySQL, PostgreSQL, Redis 等)
- 🤖 AI 驱动的安全风险分析
- 🎨 现代化深色主题界面
- ⚡ Rust 驱动的高性能扫描引擎

## 📝 版本说明

- **v1.0.1**: 修复运行时问题（推荐使用）
  - 恢复 index.html 入口脚本
  - 修复 Gemini SDK 集成
  - 移除不兼容的 Tauri v2 配置
  
- **v1.0.0**: 初始发布（存在运行时问题，不推荐使用）

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

Copyright © 2024
