# SSH 远程管理

## 1. 功能概述

OpenClawSwitch 提供完整的 SSH 远程管理功能，支持通过 SSH 协议连接远程服务器，管理远程 OpenClaw 实例的配置和运行状态。

### 1.1 核心能力

| 能力 | 说明 |
|------|------|
| SSH 连接管理 | 支持密码/密钥认证，主机密钥验证 |
| 配置文件管理 | 保存/加载/删除 SSH 连接配置 |
| 远程文件操作 | 目录浏览、文件读写 |
| 远程命令执行 | 环境检查、网关控制 |
|  Known Hosts | 主机密钥指纹持久化存储 |

### 1.2 技术架构

```
┌─────────────────────────────────────────────────────────────┐
│                    前端 SSH 界面                             │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐   │
│  │  连接对话框   │  │  配置管理页   │  │  远程文件浏览器 │   │
│  └───────────────┘  └───────────────┘  └───────────────┘   │
└─────────────────────────────────────────────────────────────┘
                            │ Tauri Commands
┌─────────────────────────────────────────────────────────────┐
│                    Rust SSH 后端                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │  ssh.rs - SSH 连接管理                                 │ │
│  │  • SshManager (连接池管理)                             │ │
│  │  • 主机密钥验证 (Known Hosts)                          │ │
│  │  • 文件操作 (SFTP)                                     │ │
│  │  • 远程命令执行                                        │ │
│  └───────────────────────────────────────────────────────┘ │
│  ┌───────────────────────────────────────────────────────┐ │
│  │  ssh_profiles.rs - 配置文件管理                        │ │
│  │  • ssh_save_profile                                    │ │
│  │  • ssh_load_profiles                                   │ │
│  │  • ssh_delete_profile                                  │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                            │ ssh2 crate
┌─────────────────────────────────────────────────────────────┐
│                    远程 SSH 服务器                           │
│  ┌───────────────────────────────────────────────────────┐ │
│  │  OpenClaw 运行时    配置文件：~/.openclaw/             │ │
│  │  Node.js 运行时     openclaw.json                      │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. SSH 连接管理

### 2.1 连接流程

```
1. 用户输入连接信息 (host/port/username)
       │
       ▼
2. 调用 ssh_connect 建立 TCP 连接
       │
       ▼
3. SSH 握手，获取主机密钥指纹
       │
       ▼
4. 检查 Known Hosts
       │
       ├── 已知主机 → 继续认证
       └── 未知主机 → 显示指纹确认对话框
       │
       ▼
5. 用户确认指纹 (如为新主机)
       │
       ▼
6. 执行认证 (密码/密钥)
       │
       ├── ssh_auth_password
       └── ssh_auth_key
       │
       ▼
7. 连接成功，保存会话状态
```

### 2.2 Tauri 命令

#### 2.2.1 连接命令

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `ssh_connect` | host, port, username | FingerprintInfo | 建立 SSH 连接，返回主机密钥信息 |
| `ssh_save_fingerprint` | fingerprint | () | 保存主机密钥到 Known Hosts |
| `ssh_disconnect` | - | () | 断开当前 SSH 连接 |

#### 2.2.2 认证命令

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `ssh_auth_password` | password | () | 密码认证 |
| `ssh_auth_key` | key_path, passphrase? | () | 密钥认证 |

#### 2.2.3 状态查询

| 命令 | 返回值 | 说明 |
|------|--------|------|
| `ssh_get_status` | SshConnectionStatus | 获取当前连接状态 |

---

## 3. 配置文件管理

### 3.1 SSH 配置文件结构

**文件路径**：`~/.openclaw/ssh_profiles.json`

```json
{
  "profiles": [
    {
      "id": "uuid-string",
      "name": "生产服务器",
      "host": "192.168.1.100",
      "port": 22,
      "username": "root",
      "auth_mode": "password",
      "password": "encrypted-password",
      "key_path": null
    },
    {
      "id": "uuid-string-2",
      "name": "测试服务器",
      "host": "192.168.1.101",
      "port": 22,
      "username": "ubuntu",
      "auth_mode": "private_key",
      "password": null,
      "key_path": "~/.ssh/id_rsa"
    }
  ]
}
```

### 3.2 SshProfile 类型定义

```typescript
interface SshProfile {
  id: string;              // 唯一标识 (UUID)
  name: string;            // 显示名称
  host: string;            // 主机地址
  port: number;            // SSH 端口 (默认 22)
  username: string;        // 用户名
  auth_mode: 'password' | 'private_key';  // 认证模式
  password?: string;       // 密码 (可选)
  key_path?: string;       // 私钥路径 (可选)
}
```

### 3.3 Tauri 命令

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `ssh_save_profile` | SshProfile | () | 保存或更新配置 |
| `ssh_load_profiles` | - | SshProfile[] | 加载所有配置 |
| `ssh_delete_profile` | id: string | () | 删除配置 |

---

## 4. 远程文件操作

### 4.1 SFTP 文件浏览

#### 4.1.1 列出目录

```rust
#[tauri::command]
pub fn ssh_list_dir(path: String) -> Result<Vec<RemoteFileEntry>, String>
```

**返回值**：
```typescript
interface RemoteFileEntry {
  name: string;     // 文件名
  path: string;     // 完整路径
  is_dir: boolean;  // 是否目录
  size: number;     // 文件大小 (字节)
}
```

#### 4.1.2 读取文件

```rust
#[tauri::command]
pub fn ssh_read_file(path: String) -> Result<String, String>
```

#### 4.1.3 写入文件

```rust
#[tauri::command]
pub fn ssh_write_file(path: String, content: String) -> Result<(), String>
```

### 4.2 配置文件搜索

```rust
#[tauri::command]
pub fn ssh_search_config() -> Result<ConfigSearchResult, String>
```

**用途**：在远程服务器上搜索 OpenClaw 配置文件

**搜索优先级**：
1. `~/.openclaw/openclaw.json`
2. `~/.openclaw/clawdbot.json`
3. `~/openclaw.json`
4. `~/clawdbot.json`

---

## 5. 远程网关控制

### 5.1 环境检查

```rust
#[tauri::command]
pub fn ssh_check_environment() -> Result<EnvironmentInfo, String>
```

**检查项**：
- Node.js 是否安装
- OpenClaw 是否安装
- 环境变量配置
- 目录权限

### 5.2 网关控制

| 命令 | 说明 |
|------|------|
| `ssh_start_gateway` | 启动远程 OpenClaw 网关 |
| `ssh_stop_gateway` | 停止远程网关 |
| `ssh_restart_gateway` | 重启远程网关 |
| `ssh_health_check` | 远程网关健康检查 |

---

## 6. Known Hosts 机制

### 6.1 文件路径

**存储位置**：`~/.openclaw/known_hosts`

### 6.2 指纹格式

支持两种指纹格式：

| 格式 | 示例 |
|------|------|
| SHA256 (Base64) | `SHA256:ABC123def456...` |
| MD5 (Hex) | `aa:bb:cc:dd:ee:ff:...` |

### 6.3 主机密钥验证流程

```
1. SSH 握手完成
       │
       ▼
2. 获取服务器主机密钥指纹
       │
       ▼
3. 查询 ~/.openclaw/known_hosts
       │
       ├── 存在 → 自动信任，继续认证
       │
       └── 不存在 → 显示确认对话框
              │
              ▼
         用户确认
              │
              ▼
         保存到 known_hosts
              │
              ▼
         继续认证
```

### 6.4 Tauri 命令

```rust
/// 保存主机密钥指纹
#[tauri::command]
pub fn ssh_save_fingerprint(fingerprint: &str) -> Result<(), String>
```

---

## 7. SshManager 连接管理

### 7.1 内部结构

```rust
pub struct SshManager {
    connection: Mutex<Option<SshConnection>>,
}

struct SshConnection {
    session: Session,      // ssh2::Session
    host: String,
    username: String,
}
```

### 7.2 连接状态管理

**单连接模式**：当前仅支持单个活动 SSH 连接

**状态流转**：
```
Disconnected ──ssh_connect──→ Connecting ──auth──→ Connected
                                     │
                              ssh_disconnect
                                     │
                                     ▼
                              Disconnected
```

### 7.3 Keepalive 机制

```rust
session.set_keepalive(true, 30); // 每 30 秒发送 keepalive
```

**作用**：
- 防止连接超时断开
- 检测连接活性

---

## 8. 前端组件

### 8.1 SshConnectModal

**用途**：SSH 连接对话框

**字段**：
- 主机地址
- 端口 (默认 22)
- 用户名
- 认证方式 (密码/密钥)
- 密码/密钥路径

### 8.2 SshFingerprintDialog

**用途**：主机密钥指纹确认对话框

**显示内容**：
- SHA256 指纹
- MD5 指纹
- 警告提示 (首次连接)

### 8.3 RemoteFileBrowser

**用途**：远程文件浏览器

**功能**：
- 目录树导航
- 文件列表展示
- 文件预览
- 文件编辑

### 8.4 SshSaveConfirmModal

**用途**：保存 SSH 配置确认对话框

---

## 9. 安全考虑

### 9.1 密码存储

**当前实现**：密码以明文存储在配置文件中

**改进建议**：
- 使用系统密钥链 (Keychain/Keyring)
- 加密存储敏感信息
- 支持仅保存连接信息，不保存密码

### 9.2 密钥认证

**推荐做法**：
- 使用 Ed25519 密钥
- 设置密钥密码短语 (passphrase)
- 限制密钥权限 (`chmod 600`)

### 9.3 主机密钥验证

**必须执行**：
- 首次连接时显示指纹供用户确认
- 指纹变化时发出警告 (防止中间人攻击)
- 提供删除已知主机的选项

---

## 10. 使用示例

### 10.1 添加 SSH 配置

```typescript
// 调用 saveConfig 保存配置
await invoke('ssh_save_profile', {
  profile: {
    id: crypto.randomUUID(),
    name: '生产服务器',
    host: '192.168.1.100',
    port: 22,
    username: 'root',
    authMode: 'password',
    password: 'secret-password'
  }
});
```

### 10.2 连接远程服务器

```typescript
// 1. 建立连接
const fingerprint = await invoke('ssh_connect', {
  host: '192.168.1.100',
  port: 22,
  username: 'root'
});

// 2. 如果是指纹未知，提示用户确认
if (!fingerprint.isKnown) {
  const confirmed = await showFingerprintDialog(fingerprint);
  if (confirmed) {
    await invoke('ssh_save_fingerprint', {
      fingerprint: fingerprint.sha256
    });
  }
}

// 3. 执行认证
await invoke('ssh_auth_password', { password: 'secret' });

// 4. 连接成功，可以开始远程操作
```

### 10.3 远程文件操作

```typescript
// 列出目录
const files = await invoke('ssh_list_dir', { path: '~/.openclaw' });

// 读取配置文件
const config = await invoke('ssh_read_file', {
  path: '~/.openclaw/openclaw.json'
});

// 写入配置
await invoke('ssh_write_file', {
  path: '~/.openclaw/openclaw.json',
  content: JSON.stringify(newConfig)
});
```

---

## 11. 错误处理

### 11.1 常见错误

| 错误 | 原因 | 解决方案 |
|------|------|----------|
| 连接超时 | 网络不通/防火墙 | 检查网络，确认端口开放 |
| 认证失败 | 密码/密钥错误 | 验证凭据，检查权限 |
| 指纹不匹配 | 主机密钥变更 | 确认服务器身份，更新 known_hosts |
| 权限拒绝 | 文件权限不足 | 检查用户权限，使用 sudo |

### 11.2 错误处理最佳实践

```typescript
try {
  await invoke('ssh_connect', { host, port, username });
} catch (error) {
  if (error.includes('连接超时')) {
    showToast('连接超时，请检查网络', 'error');
  } else if (error.includes('指纹不匹配')) {
    showSecurityWarning(error);
  } else {
    showToast(error, 'error');
  }
}
```

---

*最后更新：2026-03-19*
