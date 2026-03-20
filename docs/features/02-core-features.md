# 核心功能

## 1. 功能概览

OpenClawSwitch 提供 5 个核心功能模块，通过统一的导航系统连接：

```
┌─────────────────────────────────────────────────────────────┐
│                      导航栏 (Navigation)                     │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌────────┐ │
│  │ 工作台  │ │模型配置 │ │服务诊断 │ │消息渠道 │ │ 系统设置│ │
│  │overview │ │ai-config│ │diagnose │ │channels │ │settings│ │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## 2. 页面功能详解

### 2.1 工作台 (Overview Page)

**路径**：`/` (首页)

**功能描述**：
工作台是应用的主页面，提供网关运行状态的实时概览和快捷操作入口。

**核心组件**：

| 组件 | 功能 | 数据来源 |
|------|------|----------|
| 网关概况 | 显示网关状态、Node 运行时、配置目录、WebUI 地址 | `gatewayStore` |
| 快捷动作 | 启动/停止/重启网关、打开 WebUI | Tauri Commands |
| 运行时健康 | 显示内置 Node、OpenClaw、配置文件就绪状态 | `getRuntimeHealth()` |
| 最近动作 | 显示最近操作日志和状态 | `gatewayStore` |

**状态管理**：
```typescript
// gatewayStore
{
  status: {
    state: 'running' | 'stopped' | 'starting' | 'error',
    url: 'http://127.0.0.1:18789',
    pid: number | null
  },
  runtime: {
    nodeReady: boolean,
    openclawReady: boolean,
    configReady: boolean,
    dataDir: string
  }
}
```

**快捷操作**：
- `启动网关` - 启动内置 OpenClaw 网关进程
- `重启网关` - 重启网关并刷新状态
- `停止网关` - 停止网关进程
- `打开 OpenClaw WebUI` - 在系统浏览器中打开网关管理界面

---

### 2.2 模型配置 (Models Page)

**路径**：`/models`

**功能描述**：
管理 AI 模型提供商和模型选择，支持多个提供商配置和主备模型切换。

**核心功能**：

#### 2.2.1 提供商管理

| 操作 | 说明 | Tauri 命令 |
|------|------|-----------|
| 添加提供商 | 创建新的 API 提供商配置 | `upsert_provider` |
| 导入配置 | 从 JSON 导入完整提供商配置 | `import_provider` |
| 编辑配置 | 修改 baseURL、API Key 等 | `upsert_provider` |
| 删除提供商 | 删除提供商及其模型 | `delete_provider` |
| 获取模型列表 | 从提供商 API 拉取可用模型 | `fetch_provider_models` |

**提供商配置结构**：
```typescript
interface ProviderConfig {
  name: string;           // 提供商名称
  baseUrl: string;        // API 基础 URL
  apiKey?: string;        // API 密钥 (可选)
  api?: string;           // API 类型 (默认：openai-completions)
  models: ModelConfig[];  // 模型列表
}
```

#### 2.2.2 模型管理

| 操作 | 说明 | Tauri 命令 |
|------|------|-----------|
| 添加模型 | 手动添加模型到提供商 | `add_model_to_provider` |
| 删除模型 | 从提供商移除模型 | `remove_model_from_provider` |
| 设置主模型 | 设置主要使用的模型 | `set_primary_model` |
| 设置备胎 | 设置备用模型列表 | `set_fallback_models` |

**模型选择结构**：
```typescript
interface ModelSelection {
  primary: string;        // 主模型路径 (格式：provider/model-id)
  fallbacks: string[];    // 备用模型列表
}
```

**UI 交互**：
- 卡片式展示所有提供商
- 折叠面板管理每个提供商的模型列表
- 下拉选择设置主模型和备胎模型
- 支持从 API 动态获取模型列表

---

### 2.3 服务诊断 (Diagnostics Page)

**路径**：`/diagnostics`

**功能描述**：
提供网关服务的诊断工具和故障排查功能，帮助定位和解决运行问题。

**核心功能**：

#### 2.3.1 健康检查

| 检查项 | 说明 | 命令 |
|--------|------|------|
| 网关健康 | 检查 `localhost:18789` 是否可达 | `health_check_gateway` |
| 运行时健康 | 检查 Node 和 OpenClaw 状态 | `get_runtime_health` |
| 网关状态 | 获取网关进程详细信息 | `get_gateway_status` |

#### 2.3.2 日志查看

| 功能 | 说明 |
|------|------|
| 实时日志 | 订阅网关标准输出和错误流 |
| 日志过滤 | 按级别 (INFO/WARN/ERROR) 过滤 |
| 日志追踪 | 启动追踪日志 (`startup_trace`) |

#### 2.3.3 诊断工具

| 工具 | 功能 | 命令 |
|------|------|------|
| TUI 模式 | 打开终端进入 OpenClaw TUI | `open_tui` |
| 网关重启 | 重启网关进程 | `restart_gateway` |
| Doctor 修复 | 运行自动诊断修复工具 | `run_doctor_fix` |

**诊断流程**：
```
1. 检查网关健康状态
       ↓
2. 查看日志输出定位问题
       ↓
3. 尝试重启网关
       ↓
4. 运行 Doctor 工具自动修复
       ↓
5. 打开 TUI 进行深度调试
```

---

### 2.4 消息渠道 (Channels Page)

**路径**：`/channels`

**功能描述**：
配置和管理消息通知渠道插件，支持飞书、钉钉等平台的集成。

**支持的渠道**：

| 渠道 | 插件名 | 状态 |
|------|--------|------|
| 飞书 (Feishu) | `feishu-plugin` | ✅ 已实现 |
| 钉钉 (DingTalk) | `dingtalk-plugin` | ✅ 已实现 |
| 其他 | 可扩展 | 🔄 待实现 |

**核心功能**：

#### 2.4.1 渠道配置

| 操作 | 说明 | Tauri 命令 |
|------|------|-----------|
| 安装扩展 | 安装渠道插件到 `~/.openclaw/extensions` | `install_channel_extension` |
| 配置飞书 | 设置飞书 Webhook 和密钥 | `set_feishu_channel_config` |
| 配置钉钉 | 设置钉钉 Webhook 和密钥 | `set_dingtalk_channel_config` |
| 飞书配对 | 批准飞书配对请求 | `approve_feishu_pairing` |

#### 2.4.2 渠道能力矩阵

```typescript
interface ChannelCapability {
  id: string;                    // 渠道标识
  name: string;                  // 渠道名称
  icon: string;                  // 图标
  capabilities: {
    textMessage: boolean;        // 文本消息
    markdown: boolean;           // Markdown 格式
    image: boolean;              // 图片消息
    interactiveCard: boolean;    // 交互式卡片
    groupNotification: boolean;  // 群通知
  };
}
```

**配置流程**：
```
1. 选择渠道类型 (飞书/钉钉)
       ↓
2. 安装渠道扩展 (如未安装)
       ↓
3. 填写 Webhook URL
       ↓
4. 填写密钥 (可选)
       ↓
5. 测试连接
       ↓
6. 保存配置
```

---

### 2.5 系统设置 (Settings Page)

**路径**：`/settings`

**功能描述**：
管理系统偏好设置和应用行为配置。

**设置分类**：

#### 2.5.1 启动设置

| 选项 | 说明 | 命令 |
|------|------|------|
| 开机启动 | 系统启动时自动运行 | `get/set_launch_at_startup_enabled` |
| 启动模式 | 启动时打开的页面 | 本地存储 |

#### 2.5.2 窗口行为

| 选项 | 说明 |
|------|------|
| 最小化到托盘 | 关闭时最小化到系统托盘 |
| 窗口大小 | 自定义窗口尺寸 |
| 主题 | 亮色/暗色/系统跟随 |

#### 2.5.3 桌面偏好

| 设置 | 说明 | 命令 |
|------|------|------|
| 托盘菜单 | 配置托盘菜单项 | `get/set_desktop_preferences` |
| 通知 | 系统通知设置 | 本地存储 |

#### 2.5.4 配置管理

| 操作 | 说明 | 命令 |
|------|------|------|
| 加载配置 | 从默认目录加载 | `load_local_config` |
| 另存为 | 保存配置到新位置 | `save_config_as` |
| 打开目录 | 在文件管理器中打开配置目录 | `open_path_in_default_app` |

---

### 2.6 首次设置向导 (Setup Wizard)

**路径**：`/setup`

**功能描述**：
首次启动时的引导式设置流程，帮助用户完成初始化配置。

**设置流程**：

```
┌─────────────────────────────────────────────────────────────┐
│                    Setup Wizard Flow                        │
│                                                             │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐       │
│  │ Step 1:     │ → │ Step 2:     │ → │ Step 3:     │       │
│  │ 运行时就绪  │   │ 模型配置    │   │ 渠道设置    │       │
│  │             │   │             │   │ (可选)      │       │
│  └─────────────┘   └─────────────┘   └─────────────┘       │
│                                                             │
│                    ┌─────────────┐                          │
│                    │ Step 4:     │                          │
│                    │ 完成        │                          │
│                    └─────────────┘                          │
└─────────────────────────────────────────────────────────────┘
```

**Step 1: 运行时就绪检查**
- 检查内置 Node.js 状态
- 检查内置 OpenClaw 状态
- 检查配置文件是否存在

**Step 2: 模型配置**
- 添加第一个 AI 模型提供商
- 设置主要模型
- 可选设置备用模型

**Step 3: 渠道设置 (可选)**
- 选择是否启用消息通知
- 配置飞书或钉钉渠道

**Step 4: 完成**
- 生成默认配置
- 启动网关
- 进入主界面

---

## 3. 功能依赖关系

```
                    ┌─────────────┐
                    │ SetupPage   │
                    │ (首次设置)   │
                    └──────┬──────┘
                           │
          ┌────────────────┼────────────────┐
          │                │                │
          ▼                ▼                ▼
   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
   │OverviewPage │  │ModelsPage   │  │ChannelsPage │
   │(工作台)      │  │(模型配置)    │  │(消息渠道)    │
   └──────┬──────┘  └──────┬──────┘  └──────┬──────┘
          │                │                │
          │                │                │
          ▼                ▼                ▼
   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
   │Diagnostics  │◄─┤ SettingsPage│◄─┤  共享状态    │
   │(服务诊断)    │  │(系统设置)    │  │  (Zustand)  │
   └─────────────┘  └─────────────┘  └─────────────┘
```

## 4. Tauri 命令使用统计

| 类别 | 命令数量 | 主要用途 |
|------|----------|----------|
| 文件操作 | 7 | 配置文件加载/保存 |
| 配置操作 | 10 | 提供商和模型管理 |
| OpenClaw 工具 | 5 | 网关控制和健康检查 |
| SSH 连接 | 14 | 远程管理功能 |
| SSH 配置 | 3 | SSH 配置文件管理 |
| 安装管理 | 13 | 环境检测和依赖安装 |
| 安装后配置 | 15 | 服务安装和渠道配置 |
| 桌面偏好 | 4 | 启动项和偏好设置 |
| **总计** | **71** | - |

---

*最后更新：2026-03-19*
