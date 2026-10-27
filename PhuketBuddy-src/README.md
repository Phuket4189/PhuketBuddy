# PhuketBuddy

> macOS 适配版：请阅读 [Mac 使用与构建说明](README-macOS.md)，运行 `./build-macos.sh`。下文保留原 Windows 版本说明。

PhuketAI（`ai.pukitcore.com`）的桌面气泡助手：**余额优先**常驻桌面，点开即可查看用量与 API Key。

- 体积：单文件 **3.78 MB**，不依赖 .NET/VC 运行时（仅需系统自带 WebView2）
- 形态：72px 透明圆气泡，可拖动、松手吸边；点击展开面板
- 数据：余额、今日消耗/请求、累计消耗、近 7 日趋势、API Key 列表（可复制、可跳转管理）
- 存储：密码不落盘；令牌用 Windows DPAPI 加密，仅存本机

---

## 直接使用

双击 `PhuketBuddy.exe`（或 `src-tauri\target\release\phuketbuddy.exe`）。

分发包：`PhuketBuddy-v0.1.0-win64.zip`（1.87 MB，含 exe、说明、截图），解压即用；`release\` 目录就是解压后的样子。

首次使用：点击气泡 → 输入 PhuketAI 的邮箱和密码 → 登录后气泡即显示余额。

| 操作 | 效果 |
|---|---|
| 单击气泡 | 展开 / 收起面板 |
| 拖动气泡 | 移动；松手距屏幕边缘 ≤26px 自动吸附 |
| 右键托盘图标 | 显示气泡 / 立即刷新 / 设置 / 退出 |
| 面板右上齿轮 | 自动刷新间隔、开机自启、低余额阈值、退出登录、打开数据目录、退出程序 |
| 面板里的 Key 行 | 点击复制完整 Key |
| **充值** | 余额右侧的「充值」按钮 → 应用内打开站点购买页 |
| 「管理 API Key」「打开网站」 | 全部在应用内窗口打开对应站点页面 |

数据位置（正常情况下）：

```
%APPDATA%\PhuketBuddy\session.bin        # DPAPI 加密的令牌
%APPDATA%\PhuketBuddy\settings.json      # 设置
%APPDATA%\PhuketBuddy\cache.json         # 上次数据（用于秒开）
%APPDATA%\PhuketBuddy\raw-shape.json     # 接口字段结构（诊断用，不含数值/密钥）
%LOCALAPPDATA%\PhuketBuddy\webview\      # WebView2 缓存
```

> 若上述目录不可写（受限环境），会自动回退到 exe 同级的 `data\` 目录。

---

## 从源码构建

在 Windows 安装 Rust stable、Visual Studio C++ Build Tools 和 Windows SDK 后，在本项目目录执行：

```powershell
.\build.ps1
```

脚本使用所在目录定位项目，无需原作者的 `G:\Project` 环境。产物输出到上级 `dist/windows/PhuketBuddy.exe`，并生成 `dist/PhuketBuddy-v0.1.0-Windows.zip`。

构建架构取决于本机 Rust 工具链。Windows 通用脚本尚未在 Windows 机器实测；macOS 构建说明见 [README-macOS.md](README-macOS.md)。

### 目录结构

```
PhuketBuddy/
├── ui/                     # 前端（纯 HTML/CSS/JS，无打包器，直接内嵌进 exe）
│   ├── index.html          # 气泡窗口
│   └── panel.html          # 面板窗口（登录 / 仪表盘 / 设置）
└── src-tauri/
    ├── tauri.conf.json     # 应用配置（窗口在 lib.rs 里代码创建）
    ├── capabilities/
    ├── icons/
    └── src/
        ├── main.rs         # 入口（设置 WebView2 参数）
        ├── lib.rs          # 状态机、命令、托盘、定时刷新
        ├── api.rs          # HTTP 客户端 + 容错解析
        ├── secure.rs       # DPAPI、数据目录回退、自启
        └── win.rs          # 工作区计算、吸边、面板定位
```

---

## 接口（已实测）

| 用途 | 接口 |
|---|---|
| 登录 | `POST /api/v1/auth/login` `{email,password}` |
| 续期 | `POST /api/v1/auth/refresh` `{refresh_token}` |
| 用户/余额 | `GET /api/v1/auth/me` |
| 概览 | `GET /api/v1/usage/dashboard/stats` |
| 趋势 | `GET /api/v1/usage/dashboard/trend?start_date&end_date&granularity=day` |
| Key 列表 | `GET /api/v1/keys` |

> 站点是自有实现（非 new-api），字段名以 `raw-shape.json` 为准做防御式解析；
> 当前版本对 `balance / today_actual_cost / today_requests` 等做了多候选字段匹配。

---

## 已知限制

1. **首次真实账号联调**：仪表盘字段映射需用真实账号确认一次（登录后会自动在 `raw-shape.json` 留下字段结构）。
2. 透明窗口依赖关闭 WebView2 GPU 合成（已在 `main.rs` 固定），以兼容部分显卡驱动。
3. 未做代码签名，首次运行 SmartScreen 可能提示「更多信息 → 仍要运行」。
4. 安装包（NSIS）尚未生成；当前是绿色版单文件。

---

## 常见问题

**「管理 API Key / 打开网站」怎么打开的？**

本版**不再调用系统浏览器**，全部在**应用内窗口**里打开官网（避免浏览器权限/关联异常导致点了没反应）。

应用内窗口有独立的登录态：第一次打开会显示官网登录页，登录一次之后会记住（存在软件自己的 WebView2 数据目录里）。

> 如果你更想用系统浏览器，可以自行在浏览器里打开 `https://ai.pukitcore.com/keys`。

---

## 调试技巧

用演示数据启动（无需账号，界面显示内置示例数据）：

```powershell
$env:PHUKETBUDDY_DEMO = '1'
.\PhuketBuddy.exe
```

运行时日志：数据目录下的 `error.log`（自动 256KB 轮转）。
面板 → 设置 → 数据目录 → 「打开」可直接定位。
