export type QuickSetupStepId = 'model' | 'channel' | 'gateway'
export type QuickSetupChannelId = 'feishu' | 'dingtalk' | 'telegram' | 'discord' | 'slack'
export type QuickSetupProviderId =
  | 'dashscope-coding'
  | 'tencent-coding'
  | 'deepseek'
  | 'dashscope'
  | 'hunyuan'

export interface QuickSetupStepMeta {
  id: QuickSetupStepId
  title: string
  subtitle: string
}

export interface QuickSetupProviderPreset {
  id: QuickSetupProviderId
  name: string
  displayName: string
  description: string
  baseUrl: string
  suggestedModels: Array<{ id: string; name: string }>
}

export interface QuickSetupChannelPreset {
  id: QuickSetupChannelId
  name: string
  description: string
  placeholderLabel: string
  secretLabel: string
}

export interface GatewayInstallPlan {
  title: string
  summary: string
  commands: string[]
}

export const QUICK_SETUP_STEPS: QuickSetupStepMeta[] = [
  {
    id: 'model',
    title: '配置大模型',
    subtitle: '从预设服务商快速写入统一模型配置，并选择主模型。',
  },
  {
    id: 'channel',
    title: '配置通信渠道',
    subtitle: '用最精简的字段完成渠道接入，其他参数沿用默认值。',
  },
  {
    id: 'gateway',
    title: '安装并启动网关',
    subtitle: '按当前系统执行网关安装与启动，并等待健康检查通过。',
  },
]

export const QUICK_SETUP_PROVIDER_PRESETS: QuickSetupProviderPreset[] = [
  {
    id: 'dashscope-coding',
    name: 'dashscope-coding',
    displayName: '阿里云 Coding Plan',
    description: '阿里云百炼 Coding 专线，适合代码生成与工程场景。',
    baseUrl: 'https://coding.dashscope.aliyuncs.com/v1',
    suggestedModels: [
      { id: 'qwen3-coder-plus', name: 'Qwen3 Coder Plus' },
      { id: 'qwen3-max-2026-01-23', name: 'Qwen3 Max' },
    ],
  },
  {
    id: 'tencent-coding',
    name: 'tencent-coding',
    displayName: '腾讯云 Coding Plan',
    description: '腾讯云 LKEAP Coding 专线，兼容主流代码模型供应商。',
    baseUrl: 'https://api.lkeap.cloud.tencent.com/coding/v3',
    suggestedModels: [
      { id: 'glm-5', name: 'GLM-5' },
      { id: 'kimi-k2.5', name: 'Kimi K2.5' },
      { id: 'minimax-m2.5', name: 'MiniMax M2.5' },
    ],
  },
  {
    id: 'deepseek',
    name: 'deepseek',
    displayName: 'DeepSeek',
    description: 'DeepSeek 官方兼容接口，适合通用对话与推理。',
    baseUrl: 'https://api.deepseek.com/v1',
    suggestedModels: [
      { id: 'deepseek-chat', name: 'DeepSeek Chat' },
      { id: 'deepseek-reasoner', name: 'DeepSeek Reasoner' },
    ],
  },
  {
    id: 'dashscope',
    name: 'dashscope',
    displayName: '阿里云 DashScope',
    description: '阿里云通用兼容模式接口，适合常规模型接入。',
    baseUrl: 'https://dashscope.aliyuncs.com/compatible-mode/v1',
    suggestedModels: [
      { id: 'qwen-plus', name: 'Qwen Plus' },
      { id: 'qwen-max', name: 'Qwen Max' },
    ],
  },
  {
    id: 'hunyuan',
    name: 'hunyuan',
    displayName: '腾讯云混元',
    description: '腾讯云混元官方兼容接口，适合通用模型调用。',
    baseUrl: 'https://api.hunyuan.cloud.tencent.com/v1',
    suggestedModels: [
      { id: 'hunyuan-turbos-latest', name: 'Hunyuan Turbo S' },
      { id: 'hunyuan-t1-latest', name: 'Hunyuan T1' },
    ],
  },
]

export const QUICK_SETUP_CHANNEL_PRESETS: QuickSetupChannelPreset[] = [
  {
    id: 'feishu',
    name: '飞书',
    description: '安装扩展后仅需填写 App ID 与 App Secret。',
    placeholderLabel: 'App ID',
    secretLabel: 'App Secret',
  },
  {
    id: 'dingtalk',
    name: '钉钉',
    description: '安装扩展后仅需填写 Client ID 与 Client Secret。',
    placeholderLabel: 'Client ID',
    secretLabel: 'Client Secret',
  },
  {
    id: 'telegram',
    name: 'Telegram',
    description: '只需填写 Bot Token，保存后默认启用。',
    placeholderLabel: 'Bot Name（可选）',
    secretLabel: 'Bot Token',
  },
  {
    id: 'discord',
    name: 'Discord',
    description: '只需填写 Bot Token，保存后默认启用。',
    placeholderLabel: 'Bot Name（可选）',
    secretLabel: 'Bot Token',
  },
  {
    id: 'slack',
    name: 'Slack',
    description: '快速模式下只需填写 Bot Token，保存后默认启用。',
    placeholderLabel: 'Workspace Alias（可选）',
    secretLabel: 'Bot Token',
  },
]

export const canSkipQuickSetupStep = (stepId: QuickSetupStepId) =>
  stepId === 'model' || stepId === 'channel'

export const findProviderPreset = (presetId: string) =>
  QUICK_SETUP_PROVIDER_PRESETS.find((preset) => preset.id === presetId)

export const findChannelPreset = (channelId: string) =>
  QUICK_SETUP_CHANNEL_PRESETS.find((channel) => channel.id === channelId)

export const getGatewayInstallPlan = (os: 'windows' | 'macos' | 'linux'): GatewayInstallPlan => {
  if (os === 'windows') {
    return {
      title: 'Windows 网关服务',
      summary: '使用安装包内置的 NSSM 注册并启动 OpenClaw Gateway 服务。',
      commands: ['内置 NSSM 安装服务', '启动 OpenClaw Gateway 服务'],
    }
  }

  return {
    title: os === 'macos' ? 'macOS 网关服务' : 'Linux 网关服务',
    summary: '通过 OpenClaw CLI 安装并启动本机网关服务。',
    commands: ['openclaw gateway install', 'openclaw gateway start'],
  }
}
