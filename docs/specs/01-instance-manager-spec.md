# OpenClaw 实例管理器规格书

**版本**: v3.0.0
**状态**: 待审批
**创建日期**: 2026-03-19
**最后更新**: 2026-03-19

---

## 1. 执行摘要

### 1.1 项目愿景

将 OpenClawSwitch 从"应用内打包 OpenClaw 运行管理器"重新定位为 **"OpenClaw 实例管理控制台"**，支持三种运行模式：

| 模式 | 描述 | 使用场景 |
|------|------|----------|
| **Bundled (应用内打包)** | 使用应用内打包的 Node.js 和 OpenClaw 运行时 | 快速体验、开发调试、临时使用 |
| **Local (本地接管)** | 接管本地已安装的 OpenClaw 实例 | 生产环境、长期使用、独立部署 |
| **SSH (SSH 远程)** | 通过 SSH 管理远程服务器上的 OpenClaw 实例 | 云端部署、团队协作、集中管理 |

### 1.2 核心价值

- **灵活性**: 用户可在三种模式间无缝切换，适应不同使用场景
- **兼容性**: 保留现有 bundled 功能，同时支持本地和远程实例
- **一致性**: 统一的配置管理界面，无论实例运行在哪里
- **零冗余**: 移除 diagnostics 模块，精简代码库

### 1.3 关键变更

- **新增**: 实例状态检测系统（本地 + SSH）
- **新增**: SSH 远程命令执行和文件操作
- **新增**: 页面结构重组（侧边栏导航）
- **新增**: 安装/初始化分离的 Onboarding 流程
- **移除**: diagnostics 诊断模块
- **重构**: 配置管理统一化（本地/远程同一接口）

---

## 2. 状态检测矩阵

### 2.1 初始化检测（核心判断）

**检测目标**: 判断 OpenClaw 是否已完成初始化配置

**检测逻辑**:
```typescript
interface InitializationStatus {
  configFileExists: boolean;      // ~/.openclaw/openclaw.json 存在
  hasGatewayToken: boolean;       // gateway.token 字段存在且非空
  initialized: boolean;           // 以上两者都为 true
}
```

**检测方法**:
```rust
// 本地检测
pub fn check_local_initialization() -> Result<InitializationStatus, String> {
    let config_path = home_dir()?.join(".openclaw").join("openclaw.json");
    if !config_path.exists() {
        return Ok(InitializationStatus {
            configFileExists: false,
            hasGatewayToken: false,
            initialized: false,
        });
    }

    let content = fs::read_to_string(&config_path)?;
    let json: Value = serde_json::from_str(&content)?;
    let has_token = json.get("gateway")
        .and_then(|g| g.get("token"))
        .and_then(|t| t.as_str())
        .map(|s| !s.is_empty())
        .unwrap_or(false);

    Ok(InitializationStatus {
        configFileExists: true,
        hasGatewayToken: has_token,
        initialized: has_token,
    })
}

// SSH 远程检测
pub fn ssh_check_remote_initialization(profile_id: String) -> Result<InitializationStatus, String> {
    let ssh = get_ssh_session(&profile_id)?;

    // 检查配置文件存在
    let config_check = ssh.exec("test -f ~/.openclaw/openclaw.json && echo 'EXISTS' || echo 'MISSING'")?;
    let file_exists = config_check.trim() == "EXISTS";

    if !file_exists {
        return Ok(InitializationStatus {
            configFileExists: false,
            hasGatewayToken: false,
            initialized: false,
        });
    }

    // 读取并解析 JSON
    let content = ssh.sftp_read("~/.openclaw/openclaw.json")?;
    let json: Value = serde_json::from_str(&content)?;
    let has_token = json.get("gateway")
        .and_then(|g| g.get("token"))
        .and_then(|t| t.as_str())
        .map(|s| !s.is_empty())
        .unwrap_or(false);

    Ok(InitializationStatus {
        configFileExists: true,
        hasGatewayToken: has_token,
        initialized: has_token,
    })
}
```

### 2.2 安装检测

**检测目标**: 判断系统中是否安装了 OpenClaw

**检测逻辑**:
```typescript
interface InstallationStatus {
  installed: boolean;           // openclaw 命令可用
  version?: string;             // openclaw --version 输出
  installPath?: string;         // 命令路径
  globalInstalled: boolean;     // npm install -g 安装
  localInstalled: boolean;      // bundled 模式
}
```

**检测方法**:
```rust
// 本地安装检测
pub fn detect_local_installation() -> Result<InstallationStatus, String> {
    // Windows: where openclaw
    // macOS/Linux: which openclaw
    let cmd_output = if cfg!(windows) {
        Command::new("where").arg("openclaw").output()?
    } else {
        Command::new("which").arg("openclaw").output()?
    };

    let installed = cmd_output.status.success();
    let install_path = if installed {
        Some(String::from_utf8_lossy(&cmd_output.stdout).trim().to_string())
    } else {
        None
    };

    // 获取版本
    let version = if installed {
        Command::new("openclaw").arg("--version").output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
    } else {
        None
    };

    Ok(InstallationStatus {
        installed,
        version,
        installPath: install_path,
        globalInstalled: installed,
        localInstalled: true, // bundled 总是可用
    })
}

// SSH 远程安装检测
pub fn ssh_detect_installation(profile_id: String) -> Result<InstallationStatus, String> {
    let ssh = get_ssh_session(&profile_id)?;

    // 检查命令
    let check_cmd = if cfg!(windows) {
        "where openclaw"
    } else {
        "which openclaw"
    };

    let result = ssh.exec(check_cmd)?;
    let installed = result.trim().is_empty();

    // 获取版本
    let version = if installed {
        Some(ssh.exec("openclaw --version")?.trim().to_string())
    } else {
        None
    };

    Ok(InstallationStatus {
        installed,
        version,
        installPath: if installed { Some(result.trim().to_string()) } else { None },
        globalInstalled: installed,
        localInstalled: false,
    })
}
```

### 2.3 综合状态决策树

```
┌─────────────────────────────────────────────────────────────┐
│                    应用启动检测流程                           │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │ 检测 Bundled 模式 │
                    │ 可用性          │
                    └────────┬────────┘
                              │
         ┌────────────────────┼────────────────────┐
         │                    │                    │
         ▼                    ▼                    ▼
   ┌───────────┐        ┌───────────┐        ┌───────────┐
   │ 模式 A     │        │ 模式 B     │        │ 模式 C     │
   │ Bundled   │        │ Local     │        │ SSH       │
   │           │        │           │        │           │
   │ 使用应用内 │        │ 接管本地  │        │ 远程连接  │
   │ 运行时    │        │ 已安装实例 │        │ 实例      │
   └─────┬─────┘        └─────┬─────┘        └─────┬─────┘
         │                    │                    │
         ▼                    ▼                    ▼
   ┌─────────────────────────────────────────────────┐
   │          统一实例状态接口                        │
   │  {                                             │
   │    mode: "bundled" | "local" | "ssh",          │
   │    status: "running" | "stopped" | "error",    │
   │    version?: string,                           │
   │    config: OpenClawConfig,                     │
   │    gateway: { token?: string }                 │
   │  }                                             │
   └─────────────────────────────────────────────────┘
```

### 2.4 状态枚举

```typescript
type InstanceMode = 'bundled' | 'local' | 'ssh'

type InstanceStatus =
  | 'NOT_INSTALLED'      // 未安装（仅 local/ssh 模式）
  | 'NOT_INITIALIZED'    // 已安装但未初始化
  | 'STOPPED'            // 已初始化但网关未启动
  | 'RUNNING'            // 网关运行中
  | 'ERROR'              // 错误状态

interface OpenClawInstance {
  mode: InstanceMode
  status: InstanceStatus
  version?: string
  configPath?: string
  gateway?: {
    token?: string
    pid?: number
    port?: number
  }
  lastError?: string
}
```

---

## 3. 应用架构

### 3.1 架构图

```
┌────────────────────────────────────────────────────────────────┐
│                        前端 (React 19)                          │
├────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    路由层 (React Router)                   │  │
│  │  / → 网关状态 | /models → 模型 | /channels → 渠道 | /settings → 设置 │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              │                                   │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    状态管理 (Zustand)                      │  │
│  │  - instanceStore: 实例状态管理                              │  │
│  │  - configStore: 配置数据管理                                │  │
│  │  - uiStore: UI 状态管理                                     │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              │                                   │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    UI 组件层 (shadcn/ui)                   │  │
│  │  - Sidebar: 侧边栏导航                                     │  │
│  │  - Pages: 5 个主页面组件                                    │  │
│  │  - SetupWizard: 安装/初始化向导                            │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
                              │
                              │ Tauri Commands
                              ▼
┌────────────────────────────────────────────────────────────────┐
│                     Rust 后端 (Tauri v2.x)                      │
├────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                   实例管理器 (新增)                        │  │
│  │  - detect_local_instance()                                │  │
│  │  - detect_ssh_instance(profile_id)                        │  │
│  │  - switch_instance_mode(mode)                             │  │
│  │  - get_instance_status()                                  │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              │                                   │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────┐  │
│  │   本地实例管理     │  │   SSH 远程管理     │  │ 配置管理      │  │
│  │  - 服务控制       │  │  - SSH 连接       │  │  - 统一接口  │  │
│  │  - 健康检查       │  │  - SFTP 文件操作  │  │  - 本地读写  │  │
│  │  - 日志获取       │  │  - 远程命令执行  │  │  - 远程同步  │  │
│  └──────────────────┘  └──────────────────┘  └──────────────┘  │
│                              │                                   │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                   现有模块 (保留)                          │  │
│  │  - bundled_runtime: 打包运行时管理                         │  │
│  │  - gateway_supervisor: 网关进程监管                        │  │
│  │  - installer: 安装管理                                     │  │
│  │  - ssh_profiles: SSH 配置管理                              │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
```

### 3.2 新增 Rust 模块

#### 3.2.1 `src/instance_manager.rs`

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::AppHandle;

/// 实例运行模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InstanceMode {
    Bundled,
    Local,
    Ssh,
}

/// 实例状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InstanceStatus {
    NotInstalled,
    NotInitialized,
    Stopped,
    Running,
    Error,
}

/// OpenClaw 实例状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenClawInstance {
    pub mode: InstanceMode,
    pub status: InstanceStatus,
    pub version: Option<String>,
    pub config_path: Option<PathBuf>,
    pub gateway: Option<GatewayInfo>,
    pub last_error: Option<String>,
}

/// 网关信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayInfo {
    pub token: Option<String>,
    pub pid: Option<u32>,
    pub port: Option<u16>,
}

/// 初始化状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializationStatus {
    pub config_file_exists: bool,
    pub has_gateway_token: bool,
    pub initialized: bool,
}

/// 安装状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub install_path: Option<String>,
    pub global_installed: bool,
    pub local_installed: bool,
}

// ========== Tauri Commands ==========

/// 检测本地实例状态
#[tauri::command]
pub fn detect_local_instance() -> Result<OpenClawInstance, String> {
    let installation = detect_local_installation()?;
    let initialization = check_local_initialization()?;

    let status = if !installation.installed {
        InstanceStatus::NotInstalled
    } else if !initialization.initialized {
        InstanceStatus::NotInitialized
    } else {
        // 检查网关运行状态
        match check_gateway_status() {
            Ok(Some(_)) => InstanceStatus::Running,
            Ok(None) => InstanceStatus::Stopped,
            Err(e) => InstanceStatus::Error,
        }
    };

    Ok(OpenClawInstance {
        mode: InstanceMode::Local,
        status,
        version: installation.version,
        config_path: initialization.config_file_exists.then(|| {
            home_dir()
                .unwrap_or_default()
                .join(".openclaw")
                .join("openclaw.json")
        }),
        gateway: get_gateway_info().ok().flatten(),
        last_error: None,
    })
}

/// 检测 SSH 实例状态
#[tauri::command]
pub fn detect_ssh_instance(profile_id: String) -> Result<OpenClawInstance, String> {
    let ssh = get_ssh_session(&profile_id)?;
    let installation = ssh_detect_installation(&ssh)?;
    let initialization = ssh_check_initialization(&ssh)?;

    let status = if !installation.installed {
        InstanceStatus::NotInstalled
    } else if !initialization.initialized {
        InstanceStatus::NotInitialized
    } else {
        match ssh_check_gateway_status(&ssh) {
            Ok(true) => InstanceStatus::Running,
            Ok(false) => InstanceStatus::Stopped,
            Err(e) => InstanceStatus::Error,
        }
    };

    Ok(OpenClawInstance {
        mode: InstanceMode::Ssh,
        status,
        version: installation.version,
        config_path: None, // 远程路径不直接暴露
        gateway: ssh_get_gateway_info(&ssh).ok().flatten(),
        last_error: None,
    })
}

/// 切换实例模式
#[tauri::command]
pub fn switch_instance_mode(
    app: AppHandle,
    mode: InstanceMode,
) -> Result<OpenClawInstance, String> {
    // 保存当前模式
    let prefs = crate::desktop_prefs::get_preferences(&app)?;
    crate::desktop_prefs::save_instance_mode(&app, &mode)?;

    // 返回新模式的实例状态
    match mode {
        InstanceMode::Bundled => get_bundled_instance_status(&app),
        InstanceMode::Local => detect_local_instance(),
        InstanceMode::Ssh => {
            // SSH 模式需要 profile_id
            if let Some(profile_id) = prefs.default_ssh_profile {
                detect_ssh_instance(profile_id)
            } else {
                Err("SSH 模式需要指定 SSH 配置文件".to_string())
            }
        }
    }
}

/// 获取当前实例状态
#[tauri::command]
pub fn get_instance_status(app: AppHandle) -> Result<OpenClawInstance, String> {
    let prefs = crate::desktop_prefs::get_preferences(&app)?;
    let mode = prefs.instance_mode.unwrap_or(InstanceMode::Bundled);

    match mode {
        InstanceMode::Bundled => get_bundled_instance_status(&app),
        InstanceMode::Local => detect_local_instance(),
        InstanceMode::Ssh => {
            if let Some(profile_id) = prefs.default_ssh_profile {
                detect_ssh_instance(profile_id)
            } else {
                Err("SSH 模式未配置默认配置文件".to_string())
            }
        }
    }
}

// ========== 内部辅助函数 ==========

fn detect_local_installation() -> Result<InstallationStatus, String> {
    // 实现见 2.2 节
}

fn check_local_initialization() -> Result<InitializationStatus, String> {
    // 实现见 2.1 节
}

fn check_gateway_status() -> Result<Option<GatewayInfo>, String> {
    // 复用现有 gateway_supervisor 逻辑
}

fn get_gateway_info() -> Result<Option<GatewayInfo>, String> {
    // 复用现有 gateway_supervisor 逻辑
}

// SSH 相关辅助函数
fn get_ssh_session(profile_id: &str) -> Result<SshConnection, String> {
    // 复用现有 ssh.rs 逻辑
}

fn ssh_detect_installation(ssh: &SshConnection) -> Result<InstallationStatus, String> {
    // 实现见 2.2 节
}

fn ssh_check_initialization(ssh: &SshConnection) -> Result<InitializationStatus, String> {
    // 实现见 2.1 节
}

fn ssh_check_gateway_status(ssh: &SshConnection) -> Result<bool, String> {
    // SSH 远程检查网关状态
    let result = ssh.exec("ps aux | grep 'openclaw.*gateway' | grep -v grep")?;
    Ok(!result.trim().is_empty())
}

fn ssh_get_gateway_info(ssh: &SshConnection) -> Result<Option<GatewayInfo>, String> {
    // SSH 远程获取网关信息
    // 通过读取远程配置文件获取 token
    let content = ssh.sftp_read("~/.openclaw/openclaw.json")?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    let token = json.get("gateway")
        .and_then(|g| g.get("token"))
        .and_then(|t| t.as_str())
        .map(String::from);

    Ok(Some(GatewayInfo {
        token,
        pid: None, // SSH 模式无法直接获取 PID
        port: Some(3456), // 默认端口
    }))
}
```

#### 3.2.2 `src/ssh_remote_commands.rs` (新增)

```rust
use tauri::AppHandle;
use crate::ssh::SshConnection;

/// SSH 检查 OpenClaw 安装
#[tauri::command]
pub fn ssh_check_openclaw_installed(profile_id: String) -> Result<bool, String> {
    let ssh = get_ssh_session(&profile_id)?;

    let cmd = if cfg!(windows) {
        "where openclaw"
    } else {
        "which openclaw"
    };

    let result = ssh.exec(cmd).unwrap_or_default();
    Ok(!result.trim().is_empty())
}

/// SSH 获取 OpenClaw 版本
#[tauri::command]
pub fn ssh_get_openclaw_version(profile_id: String) -> Result<Option<String>, String> {
    let ssh = get_ssh_session(&profile_id)?;

    let version = ssh.exec("openclaw --version")?;
    Ok(Some(version.trim().to_string()))
}

/// SSH 读取 OpenClaw 配置
#[tauri::command]
pub fn ssh_read_config(profile_id: String) -> Result<String, String> {
    let ssh = get_ssh_session(&profile_id)?;
    ssh.sftp_read("~/.openclaw/openclaw.json")
}

/// SSH 写入 OpenClaw 配置
#[tauri::command]
pub fn ssh_write_config(profile_id: String, content: String) -> Result<(), String> {
    let ssh = get_ssh_session(&profile_id)?;

    // 确保目录存在
    ssh.exec("mkdir -p ~/.openclaw")?;

    // 写入配置
    ssh.sftp_write("~/.openclaw/openclaw.json", &content)?;

    Ok(())
}

/// SSH 启动网关
#[tauri::command]
pub fn ssh_start_gateway(profile_id: String) -> Result<(), String> {
    let ssh = get_ssh_session(&profile_id)?;

    // 使用 nohup 后台启动
    ssh.exec("nohup openclaw gateway start > ~/.openclaw/gateway.log 2>&1 &")?;

    Ok(())
}

/// SSH 停止网关
#[tauri::command]
pub fn ssh_stop_gateway(profile_id: String) -> Result<(), String> {
    let ssh = get_ssh_session(&profile_id)?;

    // 查找并终止进程
    ssh.exec("pkill -f 'openclaw.*gateway' || true")?;

    Ok(())
}

/// SSH 重启网关
#[tauri::command]
pub fn ssh_restart_gateway(profile_id: String) -> Result<(), String> {
    ssh_stop_gateway(profile_id.clone())?;
    std::thread::sleep(std::time::Duration::from_secs(1));
    ssh_start_gateway(profile_id)
}

/// SSH 获取网关日志
#[tauri::command]
pub fn ssh_get_gateway_logs(
    profile_id: String,
    lines: Option<usize>,
) -> Result<String, String> {
    let ssh = get_ssh_session(&profile_id)?;
    let lines = lines.unwrap_or(100);

    ssh.exec(&format!("tail -n {} ~/.openclaw/gateway.log", lines))
}

/// SSH 获取 Node.js 版本
#[tauri::command]
pub fn ssh_get_node_version(profile_id: String) -> Result<Option<String>, String> {
    let ssh = get_ssh_session(&profile_id)?;
    let version = ssh.exec("node --version")?;
    Ok(Some(version.trim().to_string()))
}

/// SSH 获取 npm 版本
#[tauri::command]
pub fn ssh_get_npm_version(profile_id: String) -> Result<Option<String>, String> {
    let ssh = get_ssh_session(&profile_id)?;
    let version = ssh.exec("npm --version")?;
    Ok(Some(version.trim().to_string()))
}

/// SSH 安装 OpenClaw
#[tauri::command]
pub fn ssh_install_openclaw(
    profile_id: String,
    registry: Option<String>,
) -> Result<String, String> {
    let ssh = get_ssh_session(&profile_id)?;

    // 检测包管理器
    let pkg_manager = ssh.exec("which pnpm || which npm || which yarn")?;
    let pkg_manager = pkg_manager.trim().split('/').last().unwrap_or("npm");

    // 构建安装命令
    let install_cmd = match registry {
        Some(reg) => format!("{} install -g @openclaw/core --registry={}", pkg_manager, reg),
        None => format!("{} install -g @openclaw/core", pkg_manager),
    };

    ssh.exec(&install_cmd)
}

/// SSH 更新 OpenClaw
#[tauri::command]
pub fn ssh_update_openclaw(
    profile_id: String,
    registry: Option<String>,
) -> Result<String, String> {
    let ssh = get_ssh_session(&profile_id)?;

    let pkg_manager = ssh.exec("which pnpm || which npm || which yarn")?;
    let pkg_manager = pkg_manager.trim().split('/').last().unwrap_or("npm");

    let update_cmd = match registry {
        Some(reg) => format!("{} update -g @openclaw/core --registry={}", pkg_manager, reg),
        None => format!("{} update -g @openclaw/core", pkg_manager),
    };

    ssh.exec(&update_cmd)
}
```

### 3.3 新增前端 State

#### 3.3.1 `src/stores/instanceStore.ts` (新增)

```typescript
import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import { invoke } from '@tauri-apps/api/core'

export type InstanceMode = 'bundled' | 'local' | 'ssh'

export type InstanceStatus =
  | 'NOT_INSTALLED'
  | 'NOT_INITIALIZED'
  | 'STOPPED'
  | 'RUNNING'
  | 'ERROR'

export interface GatewayInfo {
  token?: string
  pid?: number
  port?: number
}

export interface OpenClawInstance {
  mode: InstanceMode
  status: InstanceStatus
  version?: string
  configPath?: string
  gateway?: GatewayInfo
  lastError?: string
}

interface InstanceState {
  // 当前实例
  currentInstance: OpenClawInstance | null
  isLoading: boolean
  error: string | null

  // 操作
  loadInstance: () => Promise<void>
  switchMode: (mode: InstanceMode, profileId?: string) => Promise<void>
  startGateway: () => Promise<void>
  stopGateway: () => Promise<void>
  restartGateway: () => Promise<void>
  refresh: () => Promise<void>

  // 状态重置
  clearError: () => void
}

export const useInstanceStore = create<InstanceState>()(
  persist(
    (set, get) => ({
      currentInstance: null,
      isLoading: false,
      error: null,

      loadInstance: async () => {
        set({ isLoading: true, error: null })
        try {
          const instance = await invoke<OpenClawInstance>('get_instance_status')
          set({ currentInstance: instance, isLoading: false })
        } catch (err) {
          set({
            error: err instanceof Error ? err.message : '加载实例状态失败',
            isLoading: false
          })
        }
      },

      switchMode: async (mode: InstanceMode, profileId?: string) => {
        set({ isLoading: true, error: null })
        try {
          const instance = await invoke<OpenClawInstance>('switch_instance_mode', {
            mode,
            profileId,
          })
          set({ currentInstance: instance, isLoading: false })
        } catch (err) {
          set({
            error: err instanceof Error ? err.message : '切换实例模式失败',
            isLoading: false
          })
        }
      },

      startGateway: async () => {
        const { currentInstance } = get()
        if (!currentInstance) return

        set({ error: null })
        try {
          if (currentInstance.mode === 'bundled') {
            await invoke('start_gateway')
          } else if (currentInstance.mode === 'local') {
            await invoke('local_start_gateway')
          } else if (currentInstance.mode === 'ssh' && currentInstance.gateway) {
            await invoke('ssh_start_gateway', {
              profileId: currentInstance.gateway.token // 临时用 token 作 profile_id
            })
          }

          // 刷新状态
          await get().refresh()
        } catch (err) {
          set({
            error: err instanceof Error ? err.message : '启动网关失败'
          })
        }
      },

      stopGateway: async () => {
        const { currentInstance } = get()
        if (!currentInstance) return

        set({ error: null })
        try {
          if (currentInstance.mode === 'bundled') {
            await invoke('stop_gateway')
          } else if (currentInstance.mode === 'local') {
            await invoke('local_stop_gateway')
          } else if (currentInstance.mode === 'ssh' && currentInstance.gateway) {
            await invoke('ssh_stop_gateway', {
              profileId: currentInstance.gateway.token
            })
          }

          await get().refresh()
        } catch (err) {
          set({
            error: err instanceof Error ? err.message : '停止网关失败'
          })
        }
      },

      restartGateway: async () => {
        const { currentInstance } = get()
        if (!currentInstance) return

        set({ error: null })
        try {
          if (currentInstance.mode === 'bundled') {
            await invoke('restart_gateway')
          } else if (currentInstance.mode === 'local') {
            await invoke('local_restart_gateway')
          } else if (currentInstance.mode === 'ssh' && currentInstance.gateway) {
            await invoke('ssh_restart_gateway', {
              profileId: currentInstance.gateway.token
            })
          }

          await get().refresh()
        } catch (err) {
          set({
            error: err instanceof Error ? err.message : '重启网关失败'
          })
        }
      },

      refresh: async () => {
        await get().loadInstance()
      },

      clearError: () => set({ error: null }),
    }),
    {
      name: 'instance-storage',
      partialize: (state) => ({
        currentInstance: state.currentInstance,
      }),
    }
  )
)
```

#### 3.3.2 `src/domain/appState.ts` (重构)

```typescript
import type { InstanceStatus } from '../stores/instanceStore'

/**
 * @deprecated 使用新的 InstanceStatus 替代
 */
export type AppState =
  | 'NO_TARGET'
  | 'NEED_INSTALL'
  | 'NEED_CONFIG'
  | 'READY'
  | 'ERROR'

/**
 * 根据实例状态映射应用状态（用于兼容旧版导航逻辑）
 */
export function mapInstanceToAppState(instanceStatus: InstanceStatus | null): AppState {
  if (!instanceStatus) return 'ERROR'

  switch (instanceStatus) {
    case 'NOT_INSTALLED':
      return 'NEED_INSTALL'
    case 'NOT_INITIALIZED':
      return 'NEED_CONFIG'
    case 'RUNNING':
      return 'READY'
    case 'STOPPED':
      return 'READY' // 停止状态也是就绪，只是网关没启动
    case 'ERROR':
      return 'ERROR'
    default:
      return 'ERROR'
  }
}
```

---

## 4. 页面结构

### 4.1 整体布局

```
┌─────────────────────────────────────────────────────────────────┐
│  Header (应用标题 + 全局操作)                                      │
├────────────┬────────────────────────────────────────────────────┤
│            │                                                     │
│  Sidebar   │              Content Area                           │
│            │              (内部滚动)                              │
│  - 网关状态 │                                                     │
│  - 模型    │                                                     │
│  - 渠道    │                                                     │
│  - 设置    │                                                     │
│            │                                                     │
├────────────┴────────────────────────────────────────────────────┤
│  Status Bar (当前实例模式 | 网关状态 | 版本)                        │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 侧边栏导航

```typescript
// src/domain/navigation.ts (重构)

export type NavPage = 'gateway' | 'models' | 'channels' | 'settings'

export interface NavItem {
  id: NavPage
  label: string
  icon: string
  description?: string
}

export const NAVIGATION: NavItem[] = [
  {
    id: 'gateway',
    label: '网关状态',
    icon: 'Activity',
    description: '查看和管理 OpenClaw 网关服务',
  },
  {
    id: 'models',
    label: '模型',
    icon: 'Brain',
    description: '配置 AI 模型提供商和模型',
  },
  {
    id: 'channels',
    label: '通信渠道',
    icon: 'MessageSquare',
    description: '配置消息渠道插件',
  },
  {
    id: 'settings',
    label: '系统设置',
    icon: 'Settings',
    description: '应用和运行时配置',
  },
]
```

### 4.3 页面内容

#### 4.3.1 网关状态页 (`/`)

**功能**:
- 显示当前实例模式（Bundled / Local / SSH）
- 显示实例状态（未安装 / 未初始化 / 已停止 / 运行中 / 错误）
- 显示版本信息（OpenClaw、Node.js、npm）
- 网关控制（启动 / 停止 / 重启）
- 模式切换入口
- 实时日志查看（仅当运行中）

**组件结构**:
```tsx
// src/pages/GatewayStatus.tsx

export function GatewayStatus() {
  const { currentInstance, isLoading, switchMode, startGateway, stopGateway, restartGateway } = useInstanceStore()

  return (
    <div className="space-y-6">
      {/* 实例模式卡片 */}
      <Card>
        <CardHeader>
          <CardTitle>实例模式</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between">
            <div>
              <p className="text-lg font-semibold">
                {currentInstance?.mode === 'bundled' && '应用内打包模式'}
                {currentInstance?.mode === 'local' && '本地接管模式'}
                {currentInstance?.mode === 'ssh' && 'SSH 远程模式'}
              </p>
              <p className="text-sm text-muted-foreground">
                {getInstanceModeDescription(currentInstance?.mode)}
              </p>
            </div>
            <Button variant="outline" onClick={() => /* 打开模式切换对话框 */}>
              切换模式
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* 状态概览 */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <StatusCard
          title="OpenClaw"
          value={currentInstance?.version || '未检测到'}
          status={currentInstance?.status}
        />
        <NodeVersionCard />
        <NpmVersionCard />
        <GatewayStatusCard />
      </div>

      {/* 网关控制 */}
      <Card>
        <CardHeader>
          <CardTitle>网关控制</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex gap-2">
            <Button
              onClick={startGateway}
              disabled={currentInstance?.status === 'RUNNING'}
            >
              启动
            </Button>
            <Button
              variant="outline"
              onClick={stopGateway}
              disabled={currentInstance?.status !== 'RUNNING'}
            >
              停止
            </Button>
            <Button
              variant="outline"
              onClick={restartGateway}
              disabled={currentInstance?.status !== 'RUNNING'}
            >
              重启
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* 日志查看器 */}
      {currentInstance?.status === 'RUNNING' && (
        <LogViewer />
      )}
    </div>
  )
}
```

#### 4.3.2 模型配置页 (`/models`)

**功能**:
- 显示当前配置的模型提供商
- 添加/编辑/删除模型配置
- 快速配置预设（官方推荐配置）
- 模型测试连接

**配置结构**:
```typescript
interface ModelConfig {
  provider: 'anthropic' | 'openai' | 'gemini' | 'azure' | 'local'
  apiKey?: string
  baseUrl?: string
  models: ModelItem[]
}

interface ModelItem {
  id: string
  name: string
  enabled: boolean
  isPrimary: boolean
  contextWindow?: number
  maxTokens?: number
}
```

#### 4.3.3 通信渠道页 (`/channels`)

**功能**:
- 显示已配置的通信渠道
- 渠道插件安装状态
- 渠道配置编辑
- 快速配置预设

**渠道列表**:
```typescript
const CHANNELS = [
  { id: 'feishu', label: '飞书', plugin: '@larksuiteoapi/feishu-openclaw-plugin' },
  { id: 'wecom', label: '企业微信', plugin: '@wecom/wecom-openclaw-plugin' },
  { id: 'qq', label: 'QQ 机器人', plugin: '@sliverp/qqbot' },
  { id: 'dingtalk', label: '钉钉', plugin: '@dingtalk-real-ai/dingtalk-connector' },
  { id: 'telegram', label: 'Telegram', plugin: null }, // 内置支持
  { id: 'discord', label: 'Discord', plugin: null },
  { id: 'slack', label: 'Slack', plugin: null },
  { id: 'whatsapp', label: 'WhatsApp', plugin: null },
  { id: 'imessage', label: 'iMessage', plugin: null },
]
```

#### 4.3.4 系统设置页 (`/settings`)

**功能**:
- 实例模式管理
- SSH 配置文件管理
- 应用设置（语言、主题、通知）
- 数据管理（导入/导出配置）
- 关于

---

## 5. Onboarding 流程

### 5.1 流程图

```
┌─────────────────────────────────────────────────────────────────┐
│                      Onboarding 流程                             │
└─────────────────────────────────────────────────────────────────┘

应用首次启动
       │
       ▼
┌─────────────────┐
│ 检测当前实例状态 │
└────────┬────────┘
         │
         ├─── 状态：NOT_INSTALLED ───┐
         │                           │
         ▼                           ▼
┌─────────────────┐         ┌─────────────────┐
│ 安装模式选择     │         │ 初始化向导       │
│ - 本地安装       │         │ 1. 模型配置      │
│ - SSH 连接       │         │ 2. 渠道配置      │
│ - 使用 Bundled  │         │ 3. 系统配置      │
└────────┬────────┘         └────────┬────────┘
         │                           │
         ▼                           ▼
┌─────────────────┐         ┌─────────────────┐
│ 执行安装         │         │ 完成 ✓          │
│ 或 SSH 配置       │         │ 进入主应用       │
└────────┬────────┘         └─────────────────┘
         │
         ▼
┌─────────────────┐
│ 初始化向导       │
│ (同上)          │
└─────────────────┘
```

### 5.2 安装模式选择

```tsx
// src/components/setup/InstallationModeSelect.tsx

export function InstallationModeSelect() {
  const [selectedMode, setSelectedMode] = useState<'local' | 'ssh' | 'bundled'>('bundled')

  return (
    <div className="space-y-4">
      <h2 className="text-2xl font-bold">选择安装模式</h2>
      <p className="text-muted-foreground">
        选择 OpenClaw 的运行方式
      </p>

      <div className="grid gap-4">
        <ModeCard
          mode="bundled"
          title="应用内打包模式"
          description="使用应用内打包的 OpenClaw 运行时，无需额外安装"
          pros={['开箱即用', '无需配置', '适合快速体验']}
          cons={['版本更新需升级应用', '占用应用体积']}
          selected={selectedMode === 'bundled'}
          onSelect={() => setSelectedMode('bundled')}
        />

        <ModeCard
          mode="local"
          title="本地接管模式"
          description="接管本地已安装的 OpenClaw 实例"
          pros={['独立部署', '版本灵活', '适合生产环境']}
          cons={['需预先安装', '配置较复杂']}
          selected={selectedMode === 'local'}
          onSelect={() => setSelectedMode('local')}
        />

        <ModeCard
          mode="ssh"
          title="SSH 远程模式"
          description="管理远程服务器上的 OpenClaw 实例"
          pros={['云端部署', '团队协作', '集中管理']}
          cons={['需配置 SSH', '网络依赖']}
          selected={selectedMode === 'ssh'}
          onSelect={() => setSelectedMode('ssh')}
        />
      </div>
    </div>
  )
}
```

### 5.3 初始化向导

```tsx
// src/components/setup/SetupWizard.tsx

export function SetupWizard() {
  const [step, setStep] = useState<'installation' | 'models' | 'channels' | 'system' | 'complete'>('installation')
  const [isComplete, setIsComplete] = useState(false)

  const handleComplete = async () => {
    // 标记初始化完成
    await invoke('mark_initialization_complete')
    setIsComplete(true)
    // 跳转到主页
    navigate('/gateway')
  }

  return (
    <div className="min-h-screen bg-background">
      <div className="container max-w-4xl mx-auto py-12">
        {/* 进度条 */}
        <Stepper
          steps={[
            { id: 'installation', label: '安装' },
            { id: 'models', label: '模型' },
            { id: 'channels', label: '渠道' },
            { id: 'system', label: '系统' },
          ]}
          currentStep={step}
        />

        {/* 步骤内容 */}
        <div className="mt-8">
          {step === 'installation' && <InstallationStep />}
          {step === 'models' && <ModelsConfigStep />}
          {step === 'channels' && <ChannelsConfigStep />}
          {step === 'system' && <SystemConfigStep />}
          {step === 'complete' && <CompleteStep onComplete={handleComplete} />}
        </div>
      </div>
    </div>
  )
}
```

### 5.4 快速配置预设

```typescript
// src/lib/presets.ts

export interface QuickPreset {
  id: string
  name: string
  description: string
  config: Record<string, unknown>
}

// 模型预设
export const MODEL_PRESETS: QuickPreset[] = [
  {
    id: 'anthropic-official',
    name: 'Anthropic 官方推荐',
    description: '使用 Claude 系列模型，适合大多数场景',
    config: {
      provider: 'anthropic',
      models: [
        { id: 'claude-sonnet-4-20250514', name: 'Claude Sonnet 4', enabled: true, isPrimary: true },
        { id: 'claude-opus-4-20250514', name: 'Claude Opus 4', enabled: true, isPrimary: false },
      ],
    },
  },
  {
    id: 'openai-official',
    name: 'OpenAI 官方推荐',
    description: '使用 GPT-4 系列模型',
    config: {
      provider: 'openai',
      models: [
        { id: 'gpt-4o', name: 'GPT-4o', enabled: true, isPrimary: true },
        { id: 'gpt-4o-mini', name: 'GPT-4o Mini', enabled: true, isPrimary: false },
      ],
    },
  },
  {
    id: 'local-ollama',
    name: '本地 Ollama',
    description: '使用本地运行的 Ollama 服务',
    config: {
      provider: 'local',
      baseUrl: 'http://localhost:11434',
      models: [
        { id: 'llama3.1:8b', name: 'Llama 3.1 8B', enabled: true, isPrimary: true },
        { id: 'qwen2.5:7b', name: 'Qwen 2.5 7B', enabled: true, isPrimary: false },
      ],
    },
  },
]

// 渠道预设
export const CHANNEL_PRESETS: QuickPreset[] = [
  {
    id: 'feishu-default',
    name: '飞书默认配置',
    description: '飞书开放平台官方推荐配置',
    config: {
      appId: '',
      appSecret: '',
      encryptKey: '',
      verificationToken: '',
    },
  },
  {
    id: 'wecom-default',
    name: '企业微信默认配置',
    description: '企业微信官方推荐配置',
    config: {
      corpId: '',
      agentId: '',
      secret: '',
      token: '',
      encodingAesKey: '',
    },
  },
]

// 系统预设
export const SYSTEM_PRESETS: QuickPreset[] = [
  {
    id: 'browser-default',
    name: '浏览器默认配置',
    description: '使用系统默认浏览器',
    config: {
      browser: 'default',
      headless: false,
    },
  },
  {
    id: 'tools-default',
    name: 'Tools 默认配置',
    description: '启用所有内置工具',
    config: {
      tools: ['shell', 'editor', 'browser', 'screenshot'],
    },
  },
]
```

---

## 6. 配置文件结构

### 6.1 OpenClaw 配置文件

```typescript
// ~/.openclaw/openclaw.json

interface OpenClawConfig {
  // 网关配置
  gateway: {
    token?: string          // 网关认证令牌（初始化后生成）
    port?: number           // 网关端口（默认 3456）
    host?: string           // 网关主机（默认 localhost）
  }

  // 模型配置
  models: {
    defaultProvider?: string
    providers: {
      [key: string]: {
        apiKey?: string
        baseUrl?: string
        models: Array<{
          id: string
          name: string
          enabled: boolean
          isPrimary: boolean
        }>
      }
    }
  }

  // 渠道配置
  channels: {
    feishu?: FeishuConfig
    wecom?: WecomConfig
    qq?: QQBotConfig
    dingtalk?: DingTalkConfig
    telegram?: TelegramConfig
    // ...
  }

  // 系统配置
  system: {
    browser?: string
    language?: 'zh-CN' | 'en-US'
    theme?: 'light' | 'dark' | 'system'
    tools?: string[]
  }

  // 元数据
  meta: {
    version: string
    lastModified: string
    initializedAt?: string
  }
}
```

### 6.2 应用配置 (desktop_prefs)

```rust
// src-tauri/src/desktop_prefs.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopPreferences {
    // 实例模式
    pub instance_mode: Option<InstanceMode>,

    // SSH 相关
    pub default_ssh_profile: Option<String>,

    // UI 相关
    pub language: Option<String>,
    pub theme: Option<String>,

    // 初始化标记
    pub initialization_complete: bool,
}

impl Default for DesktopPreferences {
    fn default() -> Self {
        Self {
            instance_mode: Some(InstanceMode::Bundled),
            default_ssh_profile: None,
            language: Some("zh-CN".to_string()),
            theme: Some("system".to_string()),
            initialization_complete: false,
        }
    }
}
```

---

## 7. Tauri 命令清单

### 7.1 实例管理 (新增)

| 命令 | 参数 | 返回值 | 描述 |
|------|------|--------|------|
| `detect_local_instance` | 无 | `OpenClawInstance` | 检测本地实例状态 |
| `detect_ssh_instance` | `profile_id: String` | `OpenClawInstance` | 检测 SSH 实例状态 |
| `switch_instance_mode` | `mode: InstanceMode` | `OpenClawInstance` | 切换实例模式 |
| `get_instance_status` | 无 | `OpenClawInstance` | 获取当前实例状态 |

### 7.2 SSH 远程命令 (新增)

| 命令 | 参数 | 返回值 | 描述 |
|------|------|--------|------|
| `ssh_check_openclaw_installed` | `profile_id: String` | `bool` | SSH 检查安装 |
| `ssh_get_openclaw_version` | `profile_id: String` | `Option<String>` | SSH 获取版本 |
| `ssh_read_config` | `profile_id: String` | `String` | SSH 读取配置 |
| `ssh_write_config` | `profile_id, content: String` | `()` | SSH 写入配置 |
| `ssh_start_gateway` | `profile_id: String` | `()` | SSH 启动网关 |
| `ssh_stop_gateway` | `profile_id: String` | `()` | SSH 停止网关 |
| `ssh_restart_gateway` | `profile_id: String` | `()` | SSH 重启网关 |
| `ssh_get_gateway_logs` | `profile_id, lines: Option<usize>` | `String` | SSH 获取日志 |
| `ssh_install_openclaw` | `profile_id, registry: Option<String>` | `String` | SSH 安装 |
| `ssh_update_openclaw` | `profile_id, registry: Option<String>` | `String` | SSH 更新 |

### 7.3 现有命令 (保留)

| 分类 | 命令 | 描述 |
|------|------|------|
| **文件操作** | `read_file`, `write_file`, `file_exists`, `list_directory` | 通用文件操作 |
| **配置操作** | `read_openclaw_config`, `write_openclaw_config`, `validate_openclaw_config` | OpenClaw 配置管理 |
| **OpenClaw 工具** | `parse_openclaw_model_id`, `format_openclaw_version` | 工具函数 |
| **SSH 连接** | `ssh_connect`, `ssh_disconnect`, `ssh_test_connection` | SSH 连接管理 |
| **SSH 配置** | `ssh_save_profile`, `ssh_load_profile`, `ssh_delete_profile`, `ssh_list_profiles` | SSH 配置管理 |
| **安装管理** | `check_installation`, `install_openclaw`, `update_openclaw`, `uninstall_openclaw` | 安装管理 |
| **Post-Config** | `set_model_provider`, `set_channel_config`, `set_browser_config` | 初始化配置 |
| **桌面偏好** | `get_desktop_prefs`, `save_desktop_prefs` | 应用设置 |
| **Bundled 运行时** | `start_bundled_runtime`, `stop_bundled_runtime`, `get_bundled_status` | 打包运行时管理 |
| **网关监管** | `start_gateway`, `stop_gateway`, `restart_gateway`, `get_gateway_status`, `get_gateway_logs` | 网关进程管理 |

### 7.4 标记移除的命令

以下命令属于 diagnostics 模块，应在重构中移除：

| 命令 | 原功能 | 移除原因 |
|------|--------|----------|
| `diagnose_connection` | 诊断网络连接 | 功能冗余 |
| `diagnose_gateway` | 诊断网关状态 | 由 `get_gateway_status` 替代 |
| `diagnose_model` | 诊断模型连接 | 整合到模型测试功能 |
| `get_system_info` | 获取系统信息 | 非核心功能 |
| `get_logs` | 获取系统日志 | 由 `get_gateway_logs` 替代 |

---

## 8. 前端路由

### 8.1 路由配置

```tsx
// src/App.tsx (重构)

import { BrowserRouter, Routes, Route, Navigate } from 'react-router'
import { useInstanceStore } from './stores/instanceStore'
import { SetupWizard } from './components/setup/SetupWizard'
import { MainLayout } from './components/layout/MainLayout'
import { GatewayStatus } from './pages/GatewayStatus'
import { ModelsPage } from './pages/ModelsPage'
import { ChannelsPage } from './pages/ChannelsPage'
import { SettingsPage } from './pages/SettingsPage'

export function App() {
  const { currentInstance, isLoading } = useInstanceStore()

  // 加载中
  if (isLoading) {
    return <LoadingScreen />
  }

  // 未初始化，显示 Setup Wizard
  if (currentInstance?.status === 'NOT_INSTALLED' ||
      currentInstance?.status === 'NOT_INITIALIZED') {
    return (
      <BrowserRouter>
        <Routes>
          <Route path="/*" element={<SetupWizard />} />
        </Routes>
      </BrowserRouter>
    )
  }

  // 已初始化，显示主应用
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<MainLayout />}>
          <Route index element={<GatewayStatus />} />
          <Route path="models" element={<ModelsPage />} />
          <Route path="channels" element={<ChannelsPage />} />
          <Route path="settings" element={<SettingsPage />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Route>
      </Routes>
    </BrowserRouter>
  )
}
```

### 8.2 主布局

```tsx
// src/components/layout/MainLayout.tsx

export function MainLayout() {
  return (
    <div className="flex flex-col h-screen">
      {/* Header */}
      <header className="border-b">
        <div className="flex items-center justify-between px-4 h-14">
          <h1 className="text-lg font-semibold">OpenClaw Switch</h1>
          <GlobalActions />
        </div>
      </header>

      <div className="flex flex-1 overflow-hidden">
        {/* Sidebar */}
        <aside className="w-64 border-r">
          <SidebarNav />
        </aside>

        {/* Content */}
        <main className="flex-1 overflow-y-auto">
          <div className="p-6">
            <Outlet />
          </div>
        </main>
      </div>

      {/* Status Bar */}
      <footer className="border-t h-8 flex items-center px-4 text-xs text-muted-foreground">
        <StatusBar />
      </footer>
    </div>
  )
}
```

---

## 9. 实现计划

### 9.1 阶段划分

| 阶段 | 任务 | 预计工时 | 优先级 |
|------|------|----------|--------|
| **阶段 1** | Rust 后端 - 实例管理器 | 2 天 | P0 |
| **阶段 2** | Rust 后端 - SSH 远程命令 | 1.5 天 | P0 |
| **阶段 3** | 前端 State - instanceStore | 0.5 天 | P0 |
| **阶段 4** | 前端 - 主布局和侧边栏 | 1 天 | P0 |
| **阶段 5** | 前端 - 网关状态页 | 1 天 | P0 |
| **阶段 6** | 前端 - Setup Wizard | 1.5 天 | P1 |
| **阶段 7** | 前端 - 模型/渠道/设置页 | 2 天 | P1 |
| **阶段 8** | 清理 diagnostics 模块 | 0.5 天 | P2 |
| **阶段 9** | 测试和优化 | 1.5 天 | P1 |

**总计**: 约 11.5 个工作日

### 9.2 阶段 1: Rust 后端 - 实例管理器 (P0)

**任务**:
1. 创建 `src/instance_manager.rs` 模块
2. 实现 `OpenClawInstance`、`InstanceMode`、`InstanceStatus` 等类型
3. 实现 `detect_local_instance()` 命令
4. 实现 `detect_ssh_instance()` 命令
5. 实现 `switch_instance_mode()` 命令
6. 实现 `get_instance_status()` 命令
7. 在 `main.rs` 中注册命令
8. 集成到 `desktop_prefs` 存储

**验收标准**:
- [ ] 所有命令可通过 `invoke` 调用
- [ ] 本地实例状态检测准确
- [ ] SSH 实例状态检测准确
- [ ] 模式切换正常工作
- [ ] 偏好设置持久化

### 9.3 阶段 2: Rust 后端 - SSH 远程命令 (P0)

**任务**:
1. 创建 `src/ssh_remote_commands.rs` 模块
2. 实现所有 SSH 远程命令（见 7.2）
3. 在 `main.rs` 中注册命令
4. 编写单元测试

**验收标准**:
- [ ] 所有 SSH 命令可正常调用
- [ ] 远程配置读写正确
- [ ] 远程服务控制生效
- [ ] 错误处理完善

### 9.4 阶段 3: 前端 State - instanceStore (P0)

**任务**:
1. 创建 `src/stores/instanceStore.ts`
2. 实现 Zustand store
3. 实现 persist 中间件
4. 实现所有 action
5. 集成到 App 初始化流程

**验收标准**:
- [ ] Store 可正确加载实例状态
- [ ] 模式切换触发状态更新
- [ ] 网关控制 action 生效
- [ ] 状态持久化正常

### 9.5 阶段 4: 前端 - 主布局和侧边栏 (P0)

**任务**:
1. 创建 `src/components/layout/MainLayout.tsx`
2. 创建 `src/components/layout/SidebarNav.tsx`
3. 创建 `src/components/layout/StatusBar.tsx`
4. 创建 `src/components/layout/Header.tsx`
5. 重构路由配置

**验收标准**:
- [ ] 布局正确渲染
- [ ] 侧边栏导航正常
- [ ] 状态栏显示实例信息
- [ ] 响应式设计

### 9.6 阶段 5: 前端 - 网关状态页 (P0)

**任务**:
1. 创建 `src/pages/GatewayStatus.tsx`
2. 创建 `src/components/gateway/InstanceModeCard.tsx`
3. 创建 `src/components/gateway/StatusCards.tsx`
4. 创建 `src/components/gateway/GatewayControls.tsx`
5. 创建 `src/components/gateway/LogViewer.tsx`
6. 创建 `src/components/gateway/ModeSwitchDialog.tsx`

**验收标准**:
- [ ] 显示当前实例模式
- [ ] 显示状态卡片
- [ ] 网关控制按钮生效
- [ ] 日志实时刷新
- [ ] 模式切换对话框正常

### 9.7 阶段 6: 前端 - Setup Wizard (P1)

**任务**:
1. 创建 `src/components/setup/SetupWizard.tsx`
2. 创建 `src/components/setup/Stepper.tsx`
3. 创建 `src/components/setup/InstallationModeSelect.tsx`
4. 创建 `src/components/setup/InstallationStep.tsx`
5. 创建 `src/components/setup/ModelsConfigStep.tsx`
6. 创建 `src/components/setup/ChannelsConfigStep.tsx`
7. 创建 `src/components/setup/SystemConfigStep.tsx`
8. 创建 `src/components/setup/CompleteStep.tsx`

**验收标准**:
- [ ] 向导流程完整
- [ ] 安装模式选择生效
- [ ] 配置步骤保存正确
- [ ] 完成标记持久化

### 9.8 阶段 7: 前端 - 模型/渠道/设置页 (P1)

**任务**:
1. 创建 `src/pages/ModelsPage.tsx`
2. 创建 `src/pages/ChannelsPage.tsx`
3. 创建 `src/pages/SettingsPage.tsx`
4. 创建共享组件（配置表单、预设选择器等）

**验收标准**:
- [ ] 所有页面正常渲染
- [ ] 配置表单可编辑
- [ ] 预设快速配置生效
- [ ] 配置保存到本地/远程

### 9.9 阶段 8: 清理 diagnostics 模块 (P2)

**任务**:
1. 移除 `src/pages/DiagnosticsPage.tsx`
2. 移除导航中的 diagnostics 条目
3. 移除 Rust 端的 diagnose_* 命令
4. 更新相关类型定义
5. 清理测试文件

**验收标准**:
- [ ] 无编译错误
- [ ] 无运行时错误
- [ ] 应用正常启动

### 9.10 阶段 9: 测试和优化 (P1)

**任务**:
1. 编写单元测试（Rust）
2. 编写组件测试（TypeScript）
3. 端到端测试
4. 性能优化
5. 文档更新

**验收标准**:
- [ ] 关键路径测试覆盖 >80%
- [ ] 无严重 bug
- [ ] 文档完整

---

## 10. 技术难点评估

### 10.1 高难度 (需要深入研究)

| 难点 | 描述 | 解决思路 |
|------|------|----------|
| **SSH 远程进程管理** | 远程服务的启动/停止/状态检测 | 使用 nohup + PID 文件，或通过 systemd 管理 |
| **跨平台命令兼容** | Windows/macOS/Linux 命令差异 | 抽象命令执行层，按平台适配 |
| **配置同步冲突** | 本地/远程配置同时修改 | 乐观锁 + 版本控制，或最后写入优先 |

### 10.2 中难度 (有成熟方案)

| 难点 | 描述 | 解决思路 |
|------|------|----------|
| **实例状态检测** | 准确判断安装/初始化/运行状态 | 按优先级检测，明确决策树 |
| **网关日志实时获取** | 远程日志实时流式传输 | 分批次拉取，增量更新 |
| **模式切换状态同步** | 切换模式后状态一致性 | 切换后强制刷新，清除缓存 |

### 10.3 低难度 (直接实现)

| 难点 | 描述 | 解决思路 |
|------|------|----------|
| **前端 State 管理** | Zustand store 集成 | 遵循现有模式 |
| **UI 组件开发** | 页面和组件实现 | shadcn/ui 组件库 |
| **路由配置** | React Router 设置 | 标准配置 |

---

## 11. 风险与缓解

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| OpenClaw CLI 命令格式变更 | 高 | 中 | 调研官方文档，预留适配层 |
| SSH 远程服务管理不稳定 | 高 | 中 | 充分测试，提供多种管理方式 |
| 配置同步导致数据丢失 | 高 | 低 | 实现配置版本控制，支持回滚 |
| 跨平台兼容性问题 | 中 | 中 | 多平台测试，CI/CD 集成 |
| 开发周期超出预期 | 中 | 中 | 分阶段交付，优先 P0 任务 |

---

## 12. 附录

### 12.1 术语表

| 术语 | 定义 |
|------|------|
| **Bundled** | 应用内打包的 OpenClaw 运行时 |
| **Local** | 本地安装的 OpenClaw 实例 |
| **SSH** | 通过 SSH 远程管理的 OpenClaw 实例 |
| **Gateway** | OpenClaw 网关服务，负责 AI 模型和渠道通信 |
| **Initialization** | OpenClaw 初始化，标志是 gateway.token 存在且非空 |
| **Installation** | OpenClaw 安装，标志是 `openclaw` 命令可用 |

### 12.2 参考文档

- [OpenClaw 官方文档](https://openclaw.ai) (待调研)
- [Tauri v2 文档](https://v2.tauri.app/)
- [React Router 文档](https://reactrouter.com/)
- [Zustand 文档](https://zustand-demo.pmnd.rs/)
- [shadcn/ui 文档](https://ui.shadcn.com/)

### 12.3 相关文件

- `src-tauri/src/main.rs` - Tauri 命令入口
- `src-tauri/src/bundled_runtime.rs` - Bundled 运行时管理
- `src-tauri/src/gateway_supervisor.rs` - 网关节进程监管
- `src-tauri/src/ssh.rs` - SSH 连接管理
- `src-tauri/src/ssh_profiles.rs` - SSH 配置管理
- `src/App.tsx` - React 路由配置
- `src/domain/navigation.ts` - 导航配置
- `src/stores/` - Zustand stores

---

**文档结束**

---

## 修订历史

| 版本 | 日期 | 作者 | 变更说明 |
|------|------|------|----------|
| v1.0.0 | 2026-03-19 | AI Assistant | 初始版本 |
| v2.0.0 | 2026-03-19 | AI Assistant | 添加详细状态检测逻辑和命令定义 |
| v3.0.0 | 2026-03-19 | AI Assistant | 完整规格书，包含实现计划和风险评估 |
