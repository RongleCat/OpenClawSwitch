# 架构设计

## 1. 技术架构总览

### 1.1 整体架构图

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          OpenClawSwitch v2.0                            │
│                     Tauri v2.x + React 19 桌面应用                       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │                      前端层 (React 19 + TypeScript)                │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                │ │
│  │  │ Overview    │  │ Models      │  │ Channels    │                │ │
│  │  │ Page        │  │ Page        │  │ Page        │                │ │
│  │  └─────────────┘  └─────────────┘  └─────────────┘                │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                │ │
│  │  │ Diagnostics │  │ Settings    │  │ Setup       │                │ │
│  │  │ Page        │  │ Page        │  │ Wizard      │                │ │
│  │  └─────────────┘  └─────────────┘  └─────────────┘                │ │
│  │                                                                     │ │
│  │  ┌─────────────────────────────────────────────────────────────┐   │ │
│  │  │  组件层：shadcn/ui + Radix Primitives + 自定义组件           │   │ │
│  │  └─────────────────────────────────────────────────────────────┘   │ │
│  │  ┌─────────────────────────────────────────────────────────────┐   │ │
│  │  │  状态管理：Zustand (gatewayStore, settingsStore)            │   │ │
│  │  └─────────────────────────────────────────────────────────────┘   │ │
│  │  ┌─────────────────────────────────────────────────────────────┐   │ │
│  │  │  路由：React Router v7 | 国际化：react-i18next              │   │ │
│  │  └─────────────────────────────────────────────────────────────┘   │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                    │ Tauri Commands (IPC)             │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │                      Rust 后端层                                   │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐            │ │
│  │  │ desktop_    │  │ gateway_     │  │ installer    │            │ │
│  │  │ shell.rs    │  │ supervisor.rs│  │ .rs          │            │ │
│  │  │ 托盘/菜单/  │  │ 网关进程监控 │  │ 安装管理     │            │ │
│  │  │ 窗口管理    │  │ 和管理       │  │              │            │ │
│  │  └──────────────┘  └──────────────┘  └──────────────┘            │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐            │ │
│  │  │ ssh.rs      │  │ bundled_     │  │ startup_     │            │ │
│  │  │ SSH 连接管理  │  │ runtime.rs   │  │ trace.rs     │            │ │
│  │  │ + 配置文件   │  │ 捆绑运行时   │  │ 启动追踪     │            │ │
│  │  └──────────────┘  └──────────────┘  └──────────────┘            │ │
│  │  ┌──────────────┐  ┌──────────────┐                              │ │
│  │  │ desktop_    │  │ ssh_profiles │                              │ │
│  │  │ prefs.rs    │  │ .rs          │                              │ │
│  │  │ 桌面偏好设置 │  │ SSH 配置管理  │                              │ │
│  │  └──────────────┘  └──────────────┘                              │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                    │                                   │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │                      捆绑资源层                                    │ │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐   │ │
│  │  │ Node.js 运行时   │  │ OpenClaw 包     │  │ 渠道插件         │   │ │
│  │  │ (vendor/node)   │  │ (vendor/       │  │ (飞书/钉钉等)    │   │ │
│  │  │                 │  │  openclaw)      │  │                 │   │ │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘   │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

## 2. 分层架构说明

### 2.1 前端层 (Frontend Layer)

**技术选型**：
- React 19 - 最新稳定版本，提供并发渲染和 Suspense 支持
- TypeScript - 类型安全，提高代码质量和可维护性
- Tailwind CSS - 原子化 CSS，支持主题定制
- shadcn/ui - 基于 Radix Primitives 的无头组件库

**目录结构**：
```
src/
├── App.tsx                 # 应用根组件，路由配置
├── main.tsx                # 入口文件，初始化逻辑
├── assets/                 # 静态资源
├── components/             # 组件库
│   ├── ui/                 # shadcn/ui 基础组件
│   ├── shell/              # 应用壳组件 (布局/导航)
│   └── search/             # 全局搜索组件
├── pages/                  # 页面级组件
│   ├── OverviewPage.tsx    # 工作台概览
│   ├── ModelsPage.tsx      # AI 模型配置
│   ├── ChannelsPage.tsx    # 消息渠道配置
│   ├── DiagnosticsPage.tsx # 服务诊断
│   ├── SettingsPage.tsx    # 系统设置
│   └── SetupPage.tsx       # 首次设置向导
├── stores/                 # Zustand 状态管理
│   ├── gatewayStore.ts     # 网关状态管理
│   └── settingsStore.ts    # 设置状态管理
├── domain/                 # 业务逻辑层
│   ├── 业务模块.ts         # 各功能领域逻辑
│   └── *.test.ts           # 单元测试
├── lib/                    # 工具库
│   ├── utils.ts            # 通用工具函数
│   ├── config.ts           # 配置工具
│   ├── desktop.ts          # 桌面工具
│   └── i18n/               # 国际化
└── types/                  # TypeScript 类型定义
```

### 2.2 Rust 后端层 (Rust Backend Layer)

**核心模块**：

| 模块 | 文件 | 职责 |
|------|------|------|
| 主入口 | `main.rs` | Tauri 应用构建，命令注册 |
| 桌面 Shell | `desktop_shell.rs` | 托盘、菜单栏、窗口管理 |
| 网关监控 | `gateway_supervisor.rs` | 网关进程启动/停止/重启 |
| 安装管理 | `installer.rs` | OpenClaw 安装/卸载流程 |
| SSH 管理 | `ssh.rs` | SSH 连接、文件操作、远程命令 |
| SSH 配置 | `ssh_profiles.rs` | SSH 配置文件管理 |
| 捆绑运行时 | `bundled_runtime.rs` | Node.js 和 OpenClaw 运行时管理 |
| 桌面偏好 | `desktop_prefs.rs` | 开机启动等偏好设置 |
| 启动追踪 | `startup_trace.rs` | 启动日志追踪 |

**Tauri 命令分类**：

```
main.rs (Tauri Commands)
├── 文件操作 (5 个)
│   ├── get_default_config_path
│   ├── load_default_config / load_local_config
│   ├── load_config_from_directory
│   ├── load_config_from_file
│   ├── save_config / save_config_as
│   └── open_path_in_default_app
├── 配置操作 (10 个)
│   ├── get_providers
│   ├── get_model_selection
│   ├── set_primary_model
│   ├── set_fallback_models
│   ├── upsert_provider / import_provider
│   ├── delete_provider
│   ├── add_model_to_provider
│   ├── remove_model_from_provider
│   └── fetch_provider_models
├── OpenClaw 工具 (6 个)
│   ├── restart_gateway
│   ├── health_check_gateway
│   ├── get_runtime_health
│   ├── get_gateway_status
│   └── open_tui
├── SSH 连接 (14 个)
│   ├── ssh_connect / ssh_disconnect
│   ├── ssh_auth_password / ssh_auth_key
│   ├── ssh_get_status
│   ├── ssh_list_dir / ssh_read_file / ssh_write_file
│   ├── ssh_search_config
│   ├── ssh_check_environment
│   └── ssh_start/stop/restart_gateway
│   └── ssh_health_check
├── SSH 配置 (3 个)
│   ├── ssh_save_profile
│   ├── ssh_load_profiles
│   └── ssh_delete_profile
├── 安装管理 (13 个)
│   ├── check_openclaw/node/git/fnm_installed
│   ├── get_system_info
│   ├── detect_network_region
│   ├── check_environment
│   ├── install_fnm/node/openclaw/git
│   ├── uninstall_openclaw
│   └── run_full_install
├── 安装后配置 (15 个)
│   ├── open_terminal_with_command
│   ├── generate_default_config
│   ├── relaunch_as_admin
│   ├── install_gateway_service
│   ├── start/stop_gateway
│   ├── get/install_channel_extension
│   ├── start_openclaw_logs_follow
│   ├── is/start_openclaw_doctor
│   ├── set_feishu/dingtalk_channel_config
│   ├── approve_feishu_pairing
│   ├── open_web_ui
│   └── run_doctor_fix
└── 桌面偏好 (4 个)
    ├── get/set_desktop_preferences
    └── get/set_launch_at_startup_enabled
```

### 2.3 捆绑资源层 (Bundled Resources Layer)

**资源结构**：
```
src-tauri/resources/
├── vendor/
│   ├── node/               # Node.js 运行时
│   │   ├── node.exe        # Windows Node 可执行文件
│   │   └── lib/            # Node 核心库
│   └── openclaw/           # OpenClaw 包
│       ├── package.json    # 包描述和入口
│       ├── dist/           # 编译产物
│       └── bin/            # 命令行入口
└── .gitkeep
```

**设计特点**：
- **离线运行**：所有核心依赖捆绑，无需联网安装
- **跨平台**：Windows/macOS/Linux 分别打包对应资源
- **版本锁定**：精确控制运行时版本，避免兼容性问题

## 3. 运行模式

### 3.1 本地模式 (Local Mode)

```
┌─────────────────────────────────────────────────────┐
│                  OpenClawSwitch                      │
│  ┌────────────────────────────────────────────────┐ │
│  │              内置 Node.js 运行时                │ │
│  │              内置 OpenClaw 包                   │ │
│  └────────────────────────────────────────────────┘ │
│                      │                               │
│                      ▼                               │
│  ┌────────────────────────────────────────────────┐ │
│  │         本地网关进程 (localhost:18789)          │ │
│  └────────────────────────────────────────────────┘ │
│                      │                               │
│                      ▼                               │
│  ┌────────────────────────────────────────────────┐ │
│  │     配置文件：~/.openclaw/openclaw.json        │ │
│  └────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────┘
```

**特点**：
- 开箱即用，无需额外配置
- 所有组件本地运行
- 适合个人开发和测试

### 3.2 SSH 模式 (Remote Mode)

```
┌─────────────────────────────────────────────────────┐
│                  OpenClawSwitch                      │
│  ┌────────────────────────────────────────────────┐ │
│  │            SSH 客户端 (ssh2 crate)              │ │
│  │            SSH 配置文件管理                     │ │
│  └────────────────────────────────────────────────┘ │
│                      │ SSH 连接                     │
│                      ▼                               │
│  ┌────────────────────────────────────────────────┐ │
│  │              远程服务器                         │ │
│  │  ┌──────────────────────────────────────────┐  │ │
│  │  │    OpenClaw 网关进程                      │  │ │
│  │  │    配置文件：~/.openclaw/openclaw.json   │  │ │
│  │  └──────────────────────────────────────────┘  │ │
│  └────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────┘
```

**SSH 功能**：
- 密码/密钥认证
- 主机密钥指纹验证
- 远程文件读写
- 远程目录浏览
- 远程命令执行
- SSH 配置文件保存

## 4. 数据流设计

### 4.1 配置加载流程

```
前端发起加载请求
       │
       ▼
Rust 命令：load_local_config
       │
       ▼
检测 ~/.openclaw/openclaw.json
       │
       ▼
读取并解析 JSON → serde_json::Value
       │
       ▼
返回 (config, ConfigFileInfo) 元组
       │
       ▼
前端更新 Zustand store
       │
       ▼
UI 重新渲染
```

### 4.2 配置保存流程

```
前端修改配置
       │
       ▼
调用 save_config 命令
       │
       ▼
Rust 序列化：serde_json::to_string_pretty
       │
       ▼
写入原文件路径
       │
       ▼
返回成功/失败状态
```

### 4.3 网关启动流程

```
前端调用 start_gateway
       │
       ▼
gateway_supervisor::start_gateway
       │
       ├── 检查是否已运行 → 先停止
       │
       ▼
解析 bundled Node.js 路径
       │
       ▼
解析 bundled OpenClaw 入口
       │
       ▼
设置环境变量：
  - OPENCLAW_HOME
  - OPENCLAW_CONFIG_PATH
  - CLAWDBOT_CONFIG_PATH
  - HOME
       │
       ▼
启动子进程 (tokio::process::Command)
       │
       ▼
记录 PID 到状态
       │
       ▼
返回启动结果
```

## 5. 状态管理设计

### 5.1 Zustand Stores

**gatewayStore**：
```typescript
interface GatewayState {
  // 运行时状态
  nodeStatus: 'running' | 'stopped' | 'error'
  gatewayStatus: 'running' | 'stopped' | 'starting' | 'error'

  // 健康检查
  healthCheck: {
    webUI: boolean
    gateway: boolean
  }

  // 操作
  startGateway: () => Promise<void>
  stopGateway: () => Promise<void>
  restartGateway: () => Promise<void>
  checkHealth: () => Promise<void>
}
```

**settingsStore**：
```typescript
interface SettingsState {
  // 设置状态
  setupComplete: boolean
  loading: boolean

  // 桌面偏好
  launchAtStartup: boolean
  minimizeToTray: boolean

  // 配置信息
  configFileInfo: ConfigFileInfo | null

  // 操作
  bootstrap: () => Promise<void>
  updateSetupComplete: (complete: boolean) => void
}
```

### 5.2 状态同步机制

```
Rust 事件 ──→ Tauri Event Listener ──→ Zustand Action ──→ React Re-render
```

## 6. 关键技术决策

### 6.1 为什么选择 Tauri v2？

- **安全性**：基于系统 WebView，无内置浏览器
- **体积**：相比 Electron 小 10 倍以上
- **性能**：Rust 后端，内存占用低
- **插件系统**：官方插件支持（自动启动、托盘等）

### 6.2 为什么从 Vue 迁移到 React？

- **生态系统**：更丰富的组件库和社区支持
- **类型支持**：更好的 TypeScript 集成
- **性能**：React 19 的并发渲染特性
- **维护性**：团队更熟悉 React 技术栈

### 6.3 为什么使用 shadcn/ui？

- **无头设计**：基于 Radix Primitives，无样式预设
- **完全可控**：代码复制到项目，可自由定制
- **主题支持**：内置暗色/亮色主题切换
- **现代化**：Tailwind CSS + CSS 变量

---

*最后更新：2026-03-19*
