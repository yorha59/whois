# Whois - 局域网服务扫描器

一个基于 Rust 高性能后端和 React 现代化 UI 的局域网服务发现工具。

## 🚀 快速发布到 GitHub Release

本项目已配置 GitHub Actions 自动化流水线。只需推送标签即可自动构建并发布 macOS 和 Windows 版本。

### 发布步骤：

1. **更新版本号**：
   确保 `package.json` 和 `src-tauri/tauri.conf.json` 中的 `version` 字段是一致的（例如 `1.0.1`）。

2. **打标签并推送**：
   在终端运行：
   ```bash
   git tag v1.0.1
   git push origin v1.0.1
   ```

3. **查看进度**：
   前往 GitHub 仓库的 **Actions** 选项卡，你可以看到 "Publish Release" 工作流正在运行。

4. **下载产物**：
   构建完成后，前往 **Releases** 页面。GitHub Action 会自动创建一个草稿（Draft）发布，并上传所有构建好的安装包：
   - macOS: `.dmg`, `.app` (已压缩)
   - Windows: `.msi`, `.exe`

## 🛠️ 本地开发

参照 `README_MACOS.md` 或 `README_WINDOWS.md` 进行本地环境配置和构建。

## 🔐 隐私与安全
本应用使用 Gemini AI 进行安全分析。API Key 请通过环境变量 `process.env.API_KEY` 提供。在 GitHub Action 构建过程中，不需要该 Key。
