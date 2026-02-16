# 在 Windows 上打包 Whois

## 1. 系统要求
- **Node.js**: [下载地址](https://nodejs.org/)
- **Rust (MSVC)**: [下载地址](https://rustup.rs/) (安装时请确保选择了 Visual Studio 构建工具/C++ 桌面开发载荷)
- **WebView2**: Windows 10/11 通常自带。

## 2. 快速打包
1. 以管理员权限打开 PowerShell。
2. 进入项目根目录。
3. 执行打包脚本：
   ```powershell
   ./package_windows.ps1
   ```

## 3. 输出文件
打包完成后，产物位于：
`src-tauri/target/release/bundle/msi/Whois_1.0.0_x64_en-US.msi` (安装版)
`src-tauri/target/release/bundle/exe/Whois_1.0.0_x64.exe` (便携版)

## 4. 注意事项
- 第一次构建会比较慢，因为需要下载 Rust 编译器插件和 NSIS 工具。
- 如果遇到脚本权限问题，请先执行 `Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser`。
