
# 在 macOS 上打包 RustNet Scanner

由于本项目结合了 Rust 后端和 React 前端，推荐使用 **Tauri** 进行打包。

## 1. 准备工作
确保你的 Mac 已安装以下工具：
- **Node.js & npm/pnpm** (用于前端构建)
- **Rust** (安装命令: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)

## 2. 安装 Tauri CLI
在终端运行：
```bash
cargo install tauri-cli
```

## 3. 初始化项目结构
在当前项目根目录下运行：
```bash
cargo tauri init
```
*   **Window title**: RustNet Scanner
*   **Dist path**: `./dist` (或者你的构建输出目录)
*   **Dev server URL**: `http://localhost:5173`

## 4. 整合代码
1. 将 `scanner.rs` 中的扫描逻辑整合到 `src-tauri/src/main.rs` 中。
2. 在 `main.rs` 中使用 `#[tauri::command]` 标记扫描函数。
3. 在前端 `App.tsx` 中使用 `@tauri-apps/api` 的 `invoke` 来替换模拟的 `setInterval` 逻辑。

## 5. 打包命令
运行以下命令，Tauri 会自动编译 Rust 代码并将 React 构建出的静态资源打包进 `.app` 文件中：
```bash
# 生成静态资源 (假设你使用 vite 或类似工具)
npm run build 

# 执行打包
cargo tauri build
```
打包完成后，你可以在 `src-tauri/target/release/bundle/macos/` 目录下找到 `RustNet Scanner.app` 和 `.dmg` 安装包。

## 6. 关于图标
你可以将 1024x1024 的 PNG 图标放入根目录，运行 `cargo tauri icon your-icon.png` 自动生成 macOS 所有的图标尺寸。
