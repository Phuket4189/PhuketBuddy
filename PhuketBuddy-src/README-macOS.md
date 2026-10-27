# PhuketBuddy for macOS

此版本由提供的 Windows v0.1.0 源码适配，保留余额悬浮气泡、登录、用量趋势、API Key、站内窗口及设置。

## 安装与使用

1. 解压 `PhuketBuddy-v0.1.0-macOS-arm64.zip`。
2. 将 `PhuketBuddy.app` 拖入「应用程序」，再双击运行。
3. 单击气泡展开面板；拖动移动，靠近屏幕边缘松手自动吸附。
4. 点击菜单栏图标，可显示气泡、刷新、进入设置或退出。
5. 在设置中启用「开机自启」后，下次登录自动启动。请先移入应用程序，再开启此选项；移动应用后请关闭再重新开启。

当前产物适用于 Apple 芯片（M 系列）、macOS 11 及以上。Intel Mac 需要在 Intel Mac 上运行构建脚本另行编译。

当前采用本地临时签名，尚未使用 Apple Developer ID 签名、公证。通过微信或浏览器下载后，macOS 可能拦截：先尝试打开，再到「系统设置 → 隐私与安全性」允许此次打开。正式对外发布建议使用 Developer ID 签名并提交 Apple 公证。

## 数据与隐私

- 登录令牌：macOS 钥匙串，服务名 `com.phuketbuddy.desktop`，账户 `session`。
- 密码不保存；Windows 的 DPAPI 登录状态无法迁移，Mac 上需要重新登录。
- 设置、位置、数据缓存、日志：`~/Library/Application Support/PhuketBuddy/`。
- 完整 API Key 不写入仪表盘缓存。使用系统 WKWebView，无需安装 WebView2。
- 登录自启：`~/Library/LaunchAgents/com.phuketbuddy.desktop.plist`；关闭自启会移除该文件。

## 从源码构建

需要 Xcode Command Line Tools（`xcode-select --install`）和 Rust stable。

```sh
./build-macos.sh
```

产物在上级 `dist/` 目录。脚本编译当前 Mac 架构，创建 `.app` 并临时签名、生成 zip。可通过 `MACOS_SIGNING_IDENTITY` 指定已有的签名证书；公证须另外完成。

演示模式（不访问真实账户）：

```sh
PHUKETBUDDY_DEMO=1 PHUKETBUDDY_OPEN_PANEL=1 ../dist/PhuketBuddy.app/Contents/MacOS/PhuketBuddy
```

源码保留 Windows 的条件编译实现；构建 Mac 无需运行所附 exe。

## 验证范围

已在当前 Apple 芯片 Mac 完成 release 编译、应用签名校验，并实际启动验证演示仪表盘、设置页、点击气泡展开、Esc 收起和设置菜单退出。原生工作区读取（含屏幕外坐标回退）检查通过。

未使用真实账号测试登录、刷新令牌或付费操作；钥匙串真实账号读写、下次登录自启、实际拖动和混合缩放多屏仍需人工验收。自动化工具无法可靠拖动此类无标题浮动窗口，因此未把拖动标记为已实测。
