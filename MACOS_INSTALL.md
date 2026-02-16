# macOS 安装指南

## 📥 下载

从 [Releases 页面](https://github.com/yorha59/whois/releases) 下载 `Whois_x.x.x_aarch64.dmg` (Apple Silicon) 或 `Whois_x.x.x_x64.dmg` (Intel)

## 🔓 解决 "应用已损坏" 错误

macOS 会阻止运行未签名的应用。这是正常的安全机制。

### 方法 1: 命令行解决（推荐）

打开终端，执行：

```bash
cd ~/Downloads
xattr -cr Whois_*.dmg
```

然后双击 .dmg 文件，将 Whois 拖到 Applications 文件夹。

### 方法 2: 系统设置允许

1. 尝试打开应用
2. 出现错误提示后，打开 **系统设置 > 隐私与安全性**
3. 在底部找到 "仍要打开" 按钮
4. 点击确认

### 方法 3: 右键打开

1. 右键点击 Whois.app
2. 选择 "打开"
3. 在弹出对话框中点击 "打开"

## ✅ 验证安装

成功打开后，你会看到网络扫描界面。点击 "Start Network Scan" 开始扫描本地网络。

## 🔐 关于代码签名

此应用未经过 Apple 开发者证书签名，因此会触发 Gatekeeper 警告。这不影响应用功能，仅需要手动允许运行。

如需完全消除警告，项目维护者需要：
1. 注册 Apple Developer Program ($99/年)
2. 配置代码签名证书
3. 提交公证 (notarization)
