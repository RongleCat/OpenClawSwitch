# 安装与部署

## 1. 概述

OpenClawSwitch 提供两种部署模式：**本地模式**和**远程模式**。本地模式使用捆绑的运行时环境，开箱即用；远程模式通过 SSH 管理远程服务器。

### 1.1 安装架构

```
┌─────────────────────────────────────────────────────────────┐
│                  OpenClawSwitch 安装包                       │
│  ┌───────────────────────────────────────────────────────┐ │
│  │  主应用 (Tauri + React)                                │ │
│  │  ┌───────────────┐  ┌───────────────┐                │ │
│  │  │  Rust 后端     │  │  React 前端    │                │ │
│  │  └───────────────┘  └───────────────┘                │ │
│  └───────────────────────────────────────────────────────┘ │
│  ┌───────────────────────────────────────────────────────┐ │
│  │  捆绑资源 (src-tauri/resources/vendor/)                │ │
│  │  ┌───────────────┐  ┌───────────────┐                │ │
│  │  │  Node.js      │  │  OpenClaw     │                │ │
│  │  │  运行时        │  │  包            │                │ │
│  │  └───────────────┘  └───────────────┘                │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                            │ 安装
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                  用户系统                                     │
│  ┌───────────────────────────────────────────────────────┐ │
│  │  应用安装目录                                           │ │
│  │  Windows: %APPDATA%\com.openclawswitch.app\           │ │
│  │  macOS: ~/Library/Application Support/...             │ │
│  └───────────────────────────────────────────────────────┘ │
│  ┌───────────────────────────────────────────────────────┐ │
│  │  用户数据目录：~/.openclaw/                            │ │
│  │  ├── openclaw.json          # 配置文件                │ │
│  │  ├── known_hosts            # SSH 主机密钥            │ │
│  │  ├── ssh_profiles.json      # SSH 连接配置           │ │
│  │  ├── logs/                  # 日志目录                │ │
│  │  └── extensions/            # 渠道插件                │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. 环境检测

### 2.1 检测项目

安装前会自动检测以下环境状态：

| 检测项 | 命令 | 说明 |
|--------|------|------|
| OpenClaw | `check_openclaw_installed` | 检查是否已安装 OpenClaw |
| Node.js | `check_node_installed` | 检查 Node.js 版本 (要求 ≥ 18) |
| Git | `check_git_installed` | 检查 Git 是否安装 |
| fnm | `check_fnm_installed` | 检查 fnm (Fast Node Manager) |
| 系统信息 | `get_system_info` | OS、架构、Shell 类型 |
| 网络区域 | `detect_network_region` | 检测网络区域以选择镜像源 |

### 2.2 状态类型定义

```typescript
// OpenClaw 状态
interface OpenClawStatus {
  installed: boolean;      // 是否已安装
  version?: string;        // 版本号
  path?: string;           // 安装路径
}

// Node.js 状态
interface NodeStatus {
  installed: boolean;
  version?: string;
  meetsRequirement: boolean;  // 是否满足版本要求
}

// Git 状态
interface GitStatus {
  installed: boolean;
  version?: string;
}

// fnm 状态
interface FnmStatus {
  installed: boolean;
  version?: string;
}

// 系统信息
interface SystemInfo {
  os: string;      // 操作系统
  arch: string;    // CPU 架构
  shell: string;   // Shell 类型
}

// 综合环境状态
interface EnvironmentStatus {
  openclaw: OpenClawStatus;
  node: NodeStatus;
  git: GitStatus;
  fnm: FnmStatus;
  system: SystemInfo;
  networkRegion: string;  // 'cn' | 'global'
}
```

### 2.3 检测流程

```
1. 调用 check_environment
       │
       ▼
2. 并行检测所有依赖项
       │
       ├── check_openclaw_installed
       ├── check_node_installed
       ├── check_git_installed
       └── check_fnm_installed
       │
       ▼
3. 获取系统信息
       │
       ├── get_system_info
       └── detect_network_region
       │
       ▼
4. 返回综合检测结果
```

---

## 3. 捆绑运行时

### 3.1 资源结构

```
src-tauri/resources/vendor/
├── node/
│   ├── win32-x64/
│   │   └── node.exe
│   ├── win32-arm64/
│   │   └── node.exe
│   ├── darwin-x64/
│   │   └── bin/node
│   └── darwin-arm64/
│       └── bin/node
└── openclaw/
    ├── package.json
    ├── dist/
    └── bin/
```

### 3.2 平台标识

```rust
fn platform_target_id() -> &'static str {
    // Windows x64
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    { "win32-x64" }

    // Windows ARM64
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    { "win32-arm64" }

    // macOS x64 (Intel)
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    { "darwin-x64" }

    // macOS ARM64 (Apple Silicon)
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    { "darwin-arm64" }
}
```

### 3.3 运行时健康检查

```rust
#[tauri::command]
fn get_runtime_health(app: AppHandle) -> RuntimeHealth
```

**返回值**：
```typescript
interface RuntimeHealth {
  runtimeNotReady: boolean;   // 运行时未就绪
  nodeReady: boolean;         // Node.js 就绪
  openclawReady: boolean;     // OpenClaw 就绪
  configReady: boolean;       // 配置文件就绪
  configPath: string;         // 配置文件路径
  dataDir: string;            // 数据目录
  nodePath: string;           // Node 二进制路径
  openclawRoot: string;       // OpenClaw 根目录
}
```

---

## 4. 安装流程

### 4.1 完整安装流程

```rust
#[tauri::command]
pub fn run_full_install(app: AppHandle) -> Result<(), String>
```

**安装步骤**：

| 步骤 | 操作 | 命令 |
|------|------|------|
| 1 | 检测网络区域 | `detect_network_region` |
| 2 | 安装 fnm (如未安装) | `install_fnm` |
| 3 | 通过 fnm 安装 Node.js | `install_node_via_fnm` |
| 4 | 安装 OpenClaw | `install_openclaw` |
| 5 | 安装 Git (可选) | `install_git` |
| 6 | 生成默认配置 | `generate_default_config` |
| 7 | 安装网关服务 | `install_gateway_service` |
| 8 | 启动网关 | `start_gateway` |

### 4.2 安装进度事件

```typescript
// 进度事件
interface InstallProgressEvent {
  currentStep: number;
  totalSteps: number;
  stepName: string;
  status: 'pending' | 'running' | 'completed' | 'error';
}

// 下载进度
interface InstallDownloadEvent {
  step: string;
  percent: number;
  speed: string;       // e.g., "1.5 MB/s"
  downloaded: number;  // 已下载字节
  total: number;       // 总字节
}

// 步骤耗时
interface InstallStepTimingEvent {
  step: string;
  startTime: number;
  endTime: number;
  duration: number;    // 毫秒
}
```

### 4.3 安装日志

```typescript
interface InstallLogEvent {
  step: string;
  message: string;
  level: 'info' | 'warn' | 'error' | 'success';
  timestamp: number;
}
```

---

## 5. 依赖安装

### 5.1 安装 fnm

```rust
#[tauri::command]
pub fn install_fnm() -> Result<(), String>
```

**安装逻辑**：
1. 检测网络区域 (国内使用镜像)
2. 下载对应平台的安装脚本
3. 执行安装脚本
4. 刷新环境变量

### 5.2 安装 Node.js

```rust
#[tauri::command]
pub fn install_node_via_fnm(version: String) -> Result<(), String>
```

**参数**：
- `version`: Node.js 版本号 (如 "20.10.0")

**安装逻辑**：
1. 调用 `fnm install <version>`
2. 设置默认版本 `fnm default <version>`
3. 验证安装

### 5.3 安装 OpenClaw

```rust
#[tauri::command]
pub fn install_openclaw(
  version: String,
  registry: String
) -> Result<(), String>
```

**参数**：
- `version`: OpenClaw 版本
- `registry`: npm  registry URL

**安装逻辑**：
1. 创建安装目录
2. 使用 npm/pnpm 安装
3. 验证入口文件

### 5.4 安装 Git

```rust
#[tauri::command]
pub fn install_git() -> Result<(), String>
```

**平台差异**：
- **Windows**: 下载并运行 Git for Windows 安装包
- **macOS**: 使用 Homebrew 安装
- **Linux**: 使用包管理器安装

---

## 6. 配置管理

### 6.1 生成默认配置

```rust
#[tauri::command]
pub fn generate_default_config() -> Result<(), String>
```

**生成内容**：
```json
{
  "models": {
    "providers": {
      "default": {
        "baseUrl": "https://api.openai.com/v1",
        "api": "openai-completions",
        "models": []
      }
    }
  },
  "agents": {
    "defaults": {
      "model": {
        "primary": "default/gpt-4"
      }
    }
  }
}
```

### 6.2 配置文件路径

| 平台 | 路径 |
|------|------|
| Windows | `C:\Users\<user>\.openclaw\openclaw.json` |
| macOS | `/Users/<user>/.openclaw/openclaw.json` |
| Linux | `/home/<user>/.openclaw/openclaw.json` |

---

## 7. 服务安装

### 7.1 安装网关服务

```rust
#[tauri::command]
pub fn install_gateway_service() -> Result<(), String>
```

**Windows**：
- 使用 WiX 安装包
- 注册为 Windows 服务
- 设置开机启动

**macOS**：
- 创建 LaunchAgent
- 配置文件：`~/Library/LaunchAgents/com.openclawswitch.gateway.plist`

**Linux**：
- 创建 systemd 服务
- 配置文件：`/etc/systemd/system/openclaw-gateway.service`

### 7.2 启动/停止服务

```rust
#[tauri::command]
pub fn start_gateway() -> Result<(), String>

#[tauri::command]
pub fn stop_gateway() -> Result<(), String>
```

---

## 8. 渠道扩展安装

### 8.1 检查扩展状态

```rust
#[tauri::command]
pub fn get_channel_extension_status() -> Result<ChannelExtensionStatus, String>
```

**返回值**：
```typescript
interface ChannelExtensionStatus {
  feishuInstalled: boolean;    // 飞书插件
  wecomInstalled: boolean;     // 企业微信插件
  qqInstalled: boolean;        // QQ 插件
  dingtalkInstalled: boolean;  // 钉钉插件
}
```

### 8.2 安装扩展

```rust
#[tauri::command]
pub fn install_channel_extension(
  channel: string
) -> Result<ChannelExtensionInstallStateEvent, String>
```

**支持的渠道**：
- `feishu` - 飞书
- `dingtalk` - 钉钉
- `wecom` - 企业微信
- `qq` - QQ

### 8.3 安装位置

扩展安装到：`~/.openclaw/extensions/<channel>/`

---

## 9. Doctor 工具

### 9.1 运行诊断

```rust
#[tauri::command]
pub fn run_doctor_fix() -> Result<(), String>
```

**诊断项**：
1. 网关进程状态
2. 端口占用情况
3. 配置文件有效性
4. 依赖版本检查
5. 日志文件分析

### 9.2 自动修复

**修复操作**：
- 重启网关进程
- 重置配置文件
- 清理缓存
- 重新安装依赖

### 9.3 诊断状态

```typescript
interface DoctorStatusEvent {
  running: boolean;
  mode: 'diagnose' | 'fix';
  success?: boolean;
  exitCode?: number;
  reason?: string;
}
```

---

## 10. 日志跟踪

### 10.1 日志订阅

```rust
#[tauri::command]
pub fn start_openclaw_logs_follow() -> Result<(), String>
```

**事件类型**：
```typescript
// 实时日志
interface RealtimeLogEvent {
  message: string;
  level: 'info' | 'warn' | 'error';
  timestamp: number;
}

// 日志状态
interface RealtimeLogStatusEvent {
  running: boolean;
  reason?: string;
}
```

### 10.2 日志文件

**位置**：`~/.openclaw/logs/`

**文件命名**：
- `gateway.log` - 网关日志
- `install.log` - 安装日志
- `doctor.log` - 诊断日志

---

## 11. 管理员权限

### 11.1 以管理员身份重启

```rust
#[tauri::command]
pub fn relaunch_as_admin() -> Result<(), String>
```

**用途**：
- 安装系统服务
- 修改系统配置
- 写入受保护目录

### 11.2 平台实现

**Windows**：
- 使用 `runas` 命令
- 弹出 UAC 提示

**macOS**：
- 使用 `osascript` 请求权限
- 使用 `sudo`

**Linux**：
- 使用 `pkexec`
- 使用 `sudo`

---

## 12. 终端集成

### 12.1 打开终端

```rust
#[tauri::command]
pub fn open_terminal_with_command(command: String) -> Result<(), String>
```

**平台差异**：

| 平台 | 终端 | 命令 |
|------|------|------|
| Windows | cmd.exe | `start cmd /k <command>` |
| macOS | Terminal | `osascript -e 'tell application "Terminal" to do script "<command>"'` |
| Linux | gnome-terminal/konsole | `<terminal> -e "<command>"` |

### 12.2 打开 TUI

```rust
#[tauri::command]
pub fn open_tui(app: AppHandle) -> Result<(), String>
```

**功能**：
- 启动 OpenClaw TUI 模式
- 交互式配置
- 实时监控

---

## 13. Web UI 集成

### 13.1 打开 Web UI

```rust
#[tauri::command]
pub fn open_web_ui() -> Result<(), String>
```

**行为**：
- 在系统默认浏览器中打开
- URL: `http://127.0.0.1:18789`

### 13.2 健康检查

```rust
#[tauri::command]
pub fn health_check_gateway() -> Result<bool, String>
```

**检查**：
- TCP 连接到 `127.0.0.1:18789`
- 超时：2 秒

---

## 14. 卸载流程

### 14.1 卸载 OpenClaw

```rust
#[tauri::command]
pub fn uninstall_openclaw() -> Result<(), String>
```

**操作**：
1. 停止网关进程
2. 删除安装目录
3. 清理环境变量
4. 移除服务注册

### 14.2 保留配置

**默认保留**：
- `~/.openclaw/openclaw.json` (配置)
- `~/.openclaw/ssh_profiles.json` (SSH 配置)
- `~/.openclaw/known_hosts` (主机密钥)

---

## 15. 故障排查

### 15.1 常见问题

| 问题 | 原因 | 解决方案 |
|------|------|----------|
| 环境检测失败 | 依赖未安装 | 运行完整安装流程 |
| 网关启动失败 | 端口被占用 | 检查并关闭占用进程 |
| 配置加载失败 | JSON 格式错误 | 检查配置文件语法 |
| 服务安装失败 | 权限不足 | 以管理员身份运行 |

### 15.2 日志位置

```
~/.openclaw/logs/
├── gateway.log      # 网关运行日志
├── install.log      # 安装过程日志
├── doctor.log       # 诊断工具日志
└── error.log        # 错误日志
```

---

*最后更新：2026-03-19*
