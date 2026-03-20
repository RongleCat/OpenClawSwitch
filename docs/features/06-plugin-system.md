# 插件扩展系统

## 1. 概述

OpenClawSwitch 采用可扩展的插件架构，支持多种消息渠道的集成。插件系统允许开发者添加新的消息平台支持，而无需修改核心代码。

### 1.1 插件架构

```
┌─────────────────────────────────────────────────────────────┐
│                    OpenClaw 核心                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │  插件管理器 (Plugin Manager)                           │ │
│  │  • 插件加载                                            │ │
│  │  • 生命周期管理                                        │ │
│  │  • 配置同步                                            │ │
│  └───────────────────────────────────────────────────────┘ │
│  ┌───────────────────────────────────────────────────────┐ │
│  │  渠道抽象层 (Channel Abstraction)                      │ │
│  │  • 统一接口定义                                        │ │
│  │  • 消息路由                                            │ │
│  │  • 事件处理                                            │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                    插件层 (Plugins)                          │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│  │ 飞书插件    │ │ 钉钉插件    │ │ 企业微信插件 │           │
│  │ @larksuite  │ │ @dingtalk   │ │ @wecom      │           │
│  │ oapi/feishu │ │ -real-ai/   │ │             │           │
│  │ -openclaw   │ │ dingtalk-   │ │             │           │
│  │ -plugin     │ │ connector   │ │             │           │
│  └─────────────┘ └─────────────┘ └─────────────┘           │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│  │ QQ 插件      │ │ Telegram    │ │ Slack 插件   │           │
│  │ @sliverp/   │ │ (内置)      │ │ (内置)      │           │
│  │ qqbot       │ │             │ │             │           │
│  └─────────────┘ └─────────────┘ └─────────────┘           │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 插件分类

| 分类 | 说明 | 示例 |
|------|------|------|
| **官方插件** | 官方维护，完整支持 | 飞书、钉钉、企业微信、QQ |
| **社区插件** | 社区贡献，基本支持 | Telegram、Slack、Discord |
| **自定义插件** | 用户自建，灵活扩展 | 企业内部系统 |

---

## 2. 插件目录 (Plugin Catalog)

### 2.1 插件元数据

```typescript
interface ChannelPluginMeta {
  id: MessageChannelId;       // 渠道唯一标识
  configKey: string;           // 配置文件中的键
  displayName: string;         // 显示名称
  npmPackage: string | null;   // npm 包名
  pluginStatusKey: ChannelPluginStatusKey | null;
  supportsQuickSetup: boolean; // 支持快速设置
  supportsMessagePage: boolean; // 支持消息页面
  needsPluginInstall: boolean; // 需要安装
}
```

### 2.2 完整插件列表

```typescript
const CHANNEL_PLUGIN_CATALOG: ChannelPluginMeta[] = [
  // ────────────────────────────────────────────────────────
  // 官方插件 (需要安装)
  // ────────────────────────────────────────────────────────
  {
    id: 'feishu',
    configKey: 'feishu',
    displayName: '飞书',
    npmPackage: '@larksuiteoapi/feishu-openclaw-plugin',
    pluginStatusKey: 'feishuInstalled',
    supportsQuickSetup: true,
    supportsMessagePage: true,
    needsPluginInstall: true,
  },
  {
    id: 'wecom',
    configKey: 'wecom',
    displayName: '企业微信',
    npmPackage: '@wecom/wecom-openclaw-plugin',
    pluginStatusKey: 'wecomInstalled',
    supportsQuickSetup: true,
    supportsMessagePage: true,
    needsPluginInstall: true,
  },
  {
    id: 'qq',
    configKey: 'qqbot',
    displayName: 'QQ',
    npmPackage: '@sliverp/qqbot',
    pluginStatusKey: 'qqInstalled',
    supportsQuickSetup: true,
    supportsMessagePage: true,
    needsPluginInstall: true,
  },
  {
    id: 'dingtalk',
    configKey: 'dingtalk',
    displayName: '钉钉',
    npmPackage: '@dingtalk-real-ai/dingtalk-connector',
    pluginStatusKey: 'dingtalkInstalled',
    supportsQuickSetup: true,
    supportsMessagePage: true,
    needsPluginInstall: true,
  },

  // ────────────────────────────────────────────────────────
  // 社区插件 (内置支持，无需安装)
  // ────────────────────────────────────────────────────────
  {
    id: 'telegram',
    configKey: 'telegram',
    displayName: 'Telegram',
    npmPackage: null,
    pluginStatusKey: null,
    supportsQuickSetup: false,
    supportsMessagePage: true,
    needsPluginInstall: false,
  },
  {
    id: 'discord',
    configKey: 'discord',
    displayName: 'Discord',
    npmPackage: null,
    pluginStatusKey: null,
    supportsQuickSetup: false,
    supportsMessagePage: true,
    needsPluginInstall: false,
  },
  {
    id: 'slack',
    configKey: 'slack',
    displayName: 'Slack',
    npmPackage: null,
    pluginStatusKey: null,
    supportsQuickSetup: false,
    supportsMessagePage: true,
    needsPluginInstall: false,
  },
  {
    id: 'whatsapp',
    configKey: 'whatsapp',
    displayName: 'WhatsApp',
    npmPackage: null,
    pluginStatusKey: null,
    supportsQuickSetup: false,
    supportsMessagePage: true,
    needsPluginInstall: false,
  },
  {
    id: 'imessage',
    configKey: 'imessage',
    displayName: 'iMessage',
    npmPackage: null,
    pluginStatusKey: null,
    supportsQuickSetup: false,
    supportsMessagePage: true,
    needsPluginInstall: false,
  },
];
```

### 2.3 插件显示顺序

```typescript
// 快速设置顺序
const QUICK_SETUP_CHANNEL_ORDER = ['feishu', 'wecom', 'qq', 'dingtalk'];

// 消息页面主顺序
const MESSAGE_CHANNEL_PRIMARY_ORDER = ['feishu', 'wecom', 'qq', 'dingtalk'];

// 完整显示顺序
const MESSAGE_CHANNEL_DISPLAY_ORDER = [
  'feishu', 'wecom', 'qq', 'dingtalk',  // 官方插件
  'slack', 'whatsapp', 'imessage',      // 社区插件
  'telegram', 'discord'                  // 其他
];
```

---

## 3. 插件安装

### 3.1 安装状态检测

```typescript
interface ChannelPluginStatus {
  feishuInstalled: boolean;
  wecomInstalled: boolean;
  qqInstalled: boolean;
  dingtalkInstalled: boolean;
}
```

**检测命令**：
```rust
#[tauri::command]
pub fn get_channel_extension_status() -> Result<ChannelExtensionStatus, String>
```

### 3.2 安装流程

```
1. 用户选择渠道
       │
       ▼
2. 检查是否已安装
       │
       ├── 已安装 → 跳过
       │
       └── 未安装 → 继续
       │
       ▼
3. 调用 install_channel_extension
       │
       ▼
4. 下载 npm 包到 ~/.openclaw/extensions/
       │
       ▼
5. 更新配置启用插件
       │
       ▼
6. 返回安装状态
```

### 3.3 安装命令

```rust
#[tauri::command]
pub fn install_channel_extension(
  channel: String
) -> Result<ChannelExtensionInstallStateEvent, String>
```

**安装事件**：
```typescript
interface ChannelExtensionInstallStateEvent {
  channel: string;
  status: 'pending' | 'downloading' | 'installing' | 'completed' | 'error';
  progress?: number;
  error?: string;
}
```

### 3.4 安装位置

```
~/.openclaw/extensions/
├── feishu/
│   ├── package.json
│   ├── dist/
│   └── node_modules/
├── dingtalk/
│   ├── package.json
│   └── ...
├── wecom/
│   └── ...
└── qq/
    └── ...
```

---

## 4. 渠道配置

### 4.1 飞书渠道配置

**配置结构**：
```typescript
interface FeishuChannelConfig {
  enabled: boolean;
  appId: string;
  appSecret: string;
  domain?: string;           // 默认：https://open.feishu.cn
  connectionMode?: string;   // 连接模式
}
```

**配置命令**：
```rust
#[tauri::command]
pub fn set_feishu_channel_config(
  config: FeishuChannelConfigInput
) -> Result<(), String>
```

**配置合并逻辑**：
```typescript
// src/domain/feishuPlugin.ts
export const mergeFeishuChannelConfig = (
  root: JsonRecord,
  input: FeishuChannelConfigInput
) => {
  ensureFeishuPluginAllowed(root);  // 启用插件白名单

  const channels = ensureRecord(root, 'channels');
  const feishu = ensureRecord(channels, 'feishu');

  feishu.enabled = input.enabled;
  feishu.appId = input.appId.trim();
  feishu.appSecret = input.appSecret.trim();

  if (typeof input.domain === 'string') {
    feishu.domain = input.domain.trim();
  }
};
```

**插件白名单**：
```typescript
export const FEISHU_PLUGIN_ALLOW_ENTRY = 'feishu-openclaw-plugin';

export const ensureFeishuPluginAllowed = (root: JsonRecord) => {
  const plugins = ensureRecord(root, 'plugins');
  plugins.enabled = true;

  const allow = Array.isArray(plugins.allow)
    ? plugins.allow.filter(item => typeof item === 'string')
    : [];

  if (!allow.includes(FEISHU_PLUGIN_ALLOW_ENTRY)) {
    allow.push(FEISHU_PLUGIN_ALLOW_ENTRY);
  }

  plugins.allow = allow;
};
```

---

### 4.2 钉钉渠道配置

**配置结构**：
```typescript
interface DingtalkChannelConfig {
  enabled: boolean;
  name: string;
  clientId: string;
  clientSecret: string;
  robotCode?: string;
  corpId?: string;
  agentId?: string;
  dmPolicy?: string;        // 私信策略
  groupPolicy?: string;     // 群聊策略
  allowFrom?: string[];     // 允许来源
  messageType?: string;     // 消息类型
  cardTemplateId?: string;  // 卡片模板 ID
  debug?: boolean;
  gatewayToken?: string;
  gatewayPassword?: string;
}
```

**配置键**：
- `dingtalk` - 新版本配置键
- `dingtalk-connector` - 旧版本配置键 (兼容)

**配置命令**：
```rust
#[tauri::command]
pub fn set_dingtalk_channel_config(
  config: DingtalkChannelConfigInput
) -> Result<(), String>
```

**配置合并逻辑**：
```typescript
// src/domain/dingtalkPlugin.ts
export const DINGTALK_EDITABLE_KEYS = [
  'enabled', 'name', 'clientId', 'clientSecret',
  'robotCode', 'corpId', 'agentId',
  'dmPolicy', 'groupPolicy', 'allowFrom',
  'messageType', 'cardTemplateId', 'cardTemplateKey',
  'debug', 'gatewayToken', 'gatewayPassword',
  'sessionTimeout', 'enableMediaUpload', 'systemPrompt'
] as const;

export const mergeDingtalkEditableConfig = (
  existing: JsonRecord,
  next: JsonRecord
): JsonRecord => {
  const merged = { ...existing };

  // 先删除可编辑字段
  for (const key of DINGTALK_EDITABLE_KEYS) {
    delete merged[key];
  }

  // 再合并新值
  return { ...merged, ...next };
};
```

---

### 4.3 配置结构总览

```json
{
  "plugins": {
    "enabled": true,
    "allow": [
      "feishu-openclaw-plugin",
      "dingtalk",
      "wecom",
      "qqbot"
    ]
  },
  "channels": {
    "feishu": {
      "enabled": true,
      "appId": "cli_a1b2c3d4e5f6",
      "appSecret": "SECRET_KEY",
      "domain": "https://open.feishu.cn"
    },
    "dingtalk": {
      "enabled": true,
      "clientId": "dingxxx",
      "clientSecret": "SECRET",
      "dmPolicy": "auto_reply"
    },
    "wecom": {
      "enabled": false
    },
    "qqbot": {
      "enabled": false
    }
  }
}
```

---

## 5. 飞书配对

### 5.1 配对流程

```
1. 用户在飞书管理后台配置应用
   - 配置回调 URL
   - 获取 App ID 和 App Secret
       │
       ▼
2. 在 OpenClawSwitch 中填写配置
       │
       ▼
3. 飞书服务器发送验证请求
       │
       ▼
4. 应用返回验证 token
       │
       ▼
5. 配对成功，开始接收消息
```

### 5.2 配对批准

```rust
#[tauri::command]
pub fn approve_feishu_pairing(
  token: String,
  verify_token: String
) -> Result<(), String>
```

---

## 6. 插件能力矩阵

### 6.1 能力定义

```typescript
interface ChannelCapability {
  // 消息能力
  textMessage: boolean;       // 文本消息
  markdown: boolean;          // Markdown 格式
  richText: boolean;          // 富文本
  image: boolean;             // 图片
  file: boolean;              // 文件
  interactiveCard: boolean;   // 交互式卡片

  // 会话能力
  directMessage: boolean;     // 私信
  groupMessage: boolean;      // 群消息
  threadReply: boolean;       // 主题回复

  // 高级能力
  mention: boolean;           // @提及
  reaction: boolean;          // 表情回应
  quickReply: boolean;        // 快捷回复
  webhook: boolean;           // Webhook 推送
}
```

### 6.2 能力对比

| 渠道 | 文本 | Markdown | 图片 | 卡片 | 私信 | 群聊 |
|------|------|----------|------|------|------|------|
| 飞书 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 钉钉 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 企业微信 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| QQ | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ |
| Telegram | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ |
| Slack | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Discord | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## 7. 消息页面

### 7.1 消息页面路由

```typescript
// 支持消息页面的渠道
const MESSAGE_CHANNELS = CHANNEL_PLUGIN_CATALOG.filter(
  c => c.supportsMessagePage
);
```

### 7.2 消息页面结构

```
/ messages/:channelId
├── 消息列表
│   ├── 会话列表
│   └── 消息历史
├── 消息输入
│   ├── 文本输入
│   ├── 文件上传
│   └── 快捷操作
└── 设置面板
    ├── 渠道配置
    └── 通知设置
```

---

## 8. 自定义插件开发

### 8.1 插件接口

```typescript
interface IChannelPlugin {
  // 生命周期
  initialize(config: ChannelConfig): Promise<void>;
  shutdown(): Promise<void>;

  // 消息发送
  sendMessage(
    channelId: string,
    message: Message
  ): Promise<SendMessageResult>;

  // 消息接收
  onMessage(callback: (message: Message) => void): void;

  // 配置
  getConfigSchema(): ConfigSchema;
  validateConfig(config: ChannelConfig): ValidationResult;
}
```

### 8.2 消息类型

```typescript
interface Message {
  id: string;
  channelId: string;
  type: 'text' | 'image' | 'file' | 'card';
  content: string | MessageContent;
  sender: User;
  timestamp: number;
  metadata?: Record<string, unknown>;
}

interface User {
  id: string;
  name: string;
  avatar?: string;
}
```

### 8.3 创建插件步骤

1. **创建插件目录**
   ```bash
   mkdir my-custom-plugin
   cd my-custom-plugin
   npm init -y
   ```

2. **实现插件接口**
   ```typescript
   // src/index.ts
   export class MyCustomPlugin implements IChannelPlugin {
     async initialize(config: ChannelConfig) {
       // 初始化连接
     }

     async sendMessage(channelId: string, message: Message) {
       // 发送消息逻辑
     }

     onMessage(callback: (message: Message) => void) {
       // 监听消息
     }
   }
   ```

3. **注册插件**
   ```typescript
   // 添加到 CHANNEL_PLUGIN_CATALOG
   {
     id: 'my-custom',
     configKey: 'myCustom',
     displayName: '自定义渠道',
     npmPackage: 'my-custom-plugin',
     pluginStatusKey: null,
     supportsQuickSetup: false,
     supportsMessagePage: true,
     needsPluginInstall: true,
   }
   ```

---

## 9. 故障排查

### 9.1 常见问题

| 问题 | 原因 | 解决方案 |
|------|------|----------|
| 插件加载失败 | npm 包未安装 | 重新运行 `install_channel_extension` |
| 配置不生效 | 配置键错误 | 检查 `configKey` 是否正确 |
| 消息发送失败 | 认证失败 | 检查 App ID/Secret |
| 回调不响应 | Webhook 未配置 | 检查回调 URL 和防火墙 |

### 9.2 日志位置

```
~/.openclaw/logs/
├── plugin-feishu.log
├── plugin-dingtalk.log
├── plugin-wecom.log
└── plugin-qq.log
```

### 9.3 调试模式

在配置中启用调试：
```json
{
  "channels": {
    "feishu": {
      "debug": true
    }
  }
}
```

---

*最后更新：2026-03-19*
