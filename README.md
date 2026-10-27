# PhuketBuddy

PhuketAI 桌面悬浮工具的协作开发仓库，基于提供的 Windows v0.1.0 源码进行 macOS 适配。

## 项目入口

- [macOS 安装、构建与验证说明](PhuketBuddy-src/README-macOS.md)
- [原项目说明](PhuketBuddy-src/README.md)
- 前端：`PhuketBuddy-src/ui/`
- Tauri / Rust：`PhuketBuddy-src/src-tauri/`

## 构建 Windows 应用

需要 Rust stable、Visual Studio C++ Build Tools 和 Windows SDK。运行时需要 WebView2。

```powershell
.\PhuketBuddy-src\build.ps1
```

产物：`dist/windows/PhuketBuddy.exe` 和 `dist/PhuketBuddy-v0.1.0-Windows.zip`。构建架构由 Windows 上安装的 Rust 工具链决定。此通用 Windows 构建脚本尚未在 Windows 机器上实测。

## 构建 macOS 应用

安装 Rust stable 与 Xcode Command Line Tools 后，在仓库根目录执行：

```sh
./PhuketBuddy-src/build-macos.sh
```

生成的应用和分发包位于 `dist/`，不提交至 Git。

## 协作范围

目前包含桌面工具源码、macOS 适配、原项目设计文档和预览资源。快捷支付尚未实现；现有充值按钮仅打开站点购买页面。

生产支付密钥、服务器凭据、真实账号及用户数据不得提交到此仓库。桌面端只调用授权接口；订单校验、支付验签和到账处理应由服务端负责。

原始源码来自工具开发者；本仓库未额外授予开源许可，公开分发或更改许可前需与原作者确认。
