import type { ModelConfig, OpenClawConfig, ProviderConfig } from '../types/config'
import { QUICK_SETUP_CHANNEL_ORDER } from './channelPluginCatalog'

export type QuickSetupStepId = 'model' | 'channel' | 'gateway'
export type QuickSetupChannelId = 'feishu' | 'wecom' | 'qq' | 'dingtalk'
export type QuickSetupProviderId =
  | 'dashscope-coding'
  | 'tencent-coding'
  | 'deepseek'
  | 'dashscope'
  | 'hunyuan'
  | 'custom'

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
  isCustom?: boolean
  suggestedModels: Array<{ id: string; name: string }>
  providerModels: ModelConfig[]
}

export interface QuickSetupChannelPreset {
  id: QuickSetupChannelId
  name: string
  description: string
  placeholderLabel: string
  secretLabel?: string
}

export interface QuickSetupCustomProviderInput {
  providerName: string
  baseUrl: string
  selectedModelId: string
}

export interface QuickSetupModelOption {
  id: string
  name: string
}

export interface QuickSetupModelOptionsInput {
  fetchedModels: string[]
  modelQuery: string
}

export interface GatewayInstallPlan {
  title: string
  summary: string
  commands: string[]
}

export interface QuickSetupGatewayOptions {
  browserDefaultProfileEnabled: boolean
  toolsFullProfileEnabled: boolean
}

const QUICK_SETUP_INTERNAL_HOOK_ENTRY_IDS = [
  'boot-md',
  'bootstrap-extra-files',
  'command-logger',
  'session-memory',
] as const

export const QUICK_SETUP_PRIMARY_PROVIDER_IDS: QuickSetupProviderId[] = [
  'dashscope-coding',
  'tencent-coding',
  'deepseek',
]

const BAILIAN_MODELS: ModelConfig[] = [
  {
    id: 'qwen3.5-plus',
    name: 'qwen3.5-plus',
    reasoning: false,
    input: ['text', 'image'],
    cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
    contextWindow: 1_000_000,
    maxTokens: 65_536,
  },
  {
    id: 'qwen3-max-2026-01-23',
    name: 'qwen3-max-2026-01-23',
    reasoning: false,
    input: ['text'],
    cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
    contextWindow: 262_144,
    maxTokens: 65_536,
  },
  {
    id: 'qwen3-coder-next',
    name: 'qwen3-coder-next',
    reasoning: false,
    input: ['text'],
    cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
    contextWindow: 262_144,
    maxTokens: 65_536,
  },
  {
    id: 'qwen3-coder-plus',
    name: 'qwen3-coder-plus',
    reasoning: false,
    input: ['text'],
    cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
    contextWindow: 1_000_000,
    maxTokens: 65_536,
  },
  {
    id: 'MiniMax-M2.5',
    name: 'MiniMax-M2.5',
    reasoning: false,
    input: ['text'],
    cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
    contextWindow: 204_800,
    maxTokens: 131_072,
  },
  {
    id: 'glm-5',
    name: 'glm-5',
    reasoning: false,
    input: ['text'],
    cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
    contextWindow: 202_752,
    maxTokens: 16_384,
  },
  {
    id: 'glm-4.7',
    name: 'glm-4.7',
    reasoning: false,
    input: ['text'],
    cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
    contextWindow: 202_752,
    maxTokens: 16_384,
  },
  {
    id: 'kimi-k2.5',
    name: 'kimi-k2.5',
    reasoning: false,
    input: ['text', 'image'],
    cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
    contextWindow: 262_144,
    maxTokens: 32_768,
  },
]

const LKEAP_MODELS: ModelConfig[] = [
  {
    id: 'hunyuan-2.0-instruct',
    name: 'Tencent HY 2.0 Instruct',
    reasoning: false,
    input: ['text'],
    cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
    contextWindow: 256_000,
    maxTokens: 16_384,
  },
  {
    id: 'hunyuan-2.0-thinking',
    name: 'Tencent HY 2.0 Think',
    reasoning: true,
    input: ['text'],
    cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
    contextWindow: 256_000,
    maxTokens: 16_384,
  },
  {
    id: 'hunyuan-t1',
    name: 'Hunyuan-T1',
    reasoning: true,
    input: ['text'],
    cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
    contextWindow: 256_000,
    maxTokens: 16_384,
  },
  {
    id: 'hunyuan-turbos',
    name: 'Hunyuan-TurboS',
    reasoning: false,
    input: ['text'],
    cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
    contextWindow: 256_000,
    maxTokens: 16_384,
  },
  {
    id: 'minimax-m2.5',
    name: 'MiniMax-M2.5',
    reasoning: false,
    input: ['text'],
    cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
    contextWindow: 204_800,
    maxTokens: 131_072,
  },
  {
    id: 'kimi-k2.5',
    name: 'Kimi-K2.5',
    reasoning: false,
    input: ['text', 'image'],
    cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
    contextWindow: 262_144,
    maxTokens: 32_768,
  },
  {
    id: 'glm-5',
    name: 'glm-5',
    reasoning: false,
    input: ['text'],
    cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
    contextWindow: 202_752,
    maxTokens: 8_192,
  },
]

const cloneConfig = (config: OpenClawConfig): OpenClawConfig =>
  JSON.parse(JSON.stringify(config ?? {})) as OpenClawConfig

const buildProviderConfig = (preset: QuickSetupProviderPreset, apiKey: string, selectedModelId: string): ProviderConfig => {
  const providerModels = preset.providerModels.map((model) => ({ ...model }))

  if (!providerModels.some((model) => model.id === selectedModelId)) {
    providerModels.push({
      id: selectedModelId,
      name: selectedModelId,
    })
  }

  return {
    baseUrl: preset.baseUrl,
    apiKey,
    api: 'openai-completions',
    models: providerModels,
  }
}

export const QUICK_SETUP_STEPS: QuickSetupStepMeta[] = [
  {
    id: 'model',
    title: '配置大模型',
    subtitle: '选择服务商、填写 Key，并确认一个主模型。',
  },
  {
    id: 'channel',
    title: '配置通信渠道',
    subtitle: '快速接入一个消息渠道。',
  },
  {
    id: 'gateway',
    title: '安装并启动网关',
    subtitle: '按当前系统完成安装并等待网关就绪。',
  },
]

export const QUICK_SETUP_PROVIDER_PRESETS: QuickSetupProviderPreset[] = [
  {
    id: 'dashscope-coding',
    name: 'bailian',
    displayName: '阿里云 Coding',
    description: '阿里云 Coding 兼容 OpenAI Completions。',
    baseUrl: 'https://coding.dashscope.aliyuncs.com/v1',
    suggestedModels: [
      { id: 'qwen3.5-plus', name: 'qwen3.5-plus' },
      { id: 'qwen3-coder-plus', name: 'qwen3-coder-plus' },
      { id: 'qwen3-max-2026-01-23', name: 'qwen3-max-2026-01-23' },
    ],
    providerModels: BAILIAN_MODELS,
  },
  {
    id: 'tencent-coding',
    name: 'lkeap',
    displayName: '腾讯云 Coding',
    description: '腾讯云 Coding 兼容 OpenAI Completions。',
    baseUrl: 'https://api.lkeap.cloud.tencent.com/coding/v3',
    suggestedModels: [
      { id: 'hunyuan-2.0-instruct', name: 'Tencent HY 2.0 Instruct' },
      { id: 'hunyuan-2.0-thinking', name: 'Tencent HY 2.0 Think' },
      { id: 'hunyuan-t1', name: 'Hunyuan-T1' },
      { id: 'hunyuan-turbos', name: 'Hunyuan-TurboS' },
      { id: 'minimax-m2.5', name: 'MiniMax-M2.5' },
      { id: 'kimi-k2.5', name: 'Kimi-K2.5' },
      { id: 'glm-5', name: 'GLM-5' },
    ],
    providerModels: LKEAP_MODELS,
  },
  {
    id: 'deepseek',
    name: 'deepseek',
    displayName: 'DeepSeek',
    description: 'DeepSeek 官方兼容接口。',
    baseUrl: 'https://api.deepseek.com/v1',
    suggestedModels: [
      { id: 'deepseek-chat', name: 'DeepSeek Chat' },
      { id: 'deepseek-reasoner', name: 'DeepSeek Reasoner' },
    ],
    providerModels: [
      { id: 'deepseek-chat', name: 'DeepSeek Chat' },
      { id: 'deepseek-reasoner', name: 'DeepSeek Reasoner' },
    ],
  },
  {
    id: 'dashscope',
    name: 'dashscope',
    displayName: '阿里云 DashScope',
    description: '阿里云通用兼容模式。',
    baseUrl: 'https://dashscope.aliyuncs.com/compatible-mode/v1',
    suggestedModels: [
      { id: 'qwen-plus', name: 'Qwen Plus' },
      { id: 'qwen-max', name: 'Qwen Max' },
    ],
    providerModels: [
      { id: 'qwen-plus', name: 'Qwen Plus' },
      { id: 'qwen-max', name: 'Qwen Max' },
    ],
  },
  {
    id: 'hunyuan',
    name: 'hunyuan',
    displayName: '腾讯云混元',
    description: '腾讯云混元官方兼容接口。',
    baseUrl: 'https://api.hunyuan.cloud.tencent.com/v1',
    suggestedModels: [
      { id: 'hunyuan-turbos-latest', name: 'Hunyuan Turbo S' },
      { id: 'hunyuan-t1-latest', name: 'Hunyuan T1' },
    ],
    providerModels: [
      { id: 'hunyuan-turbos-latest', name: 'Hunyuan Turbo S' },
      { id: 'hunyuan-t1-latest', name: 'Hunyuan T1' },
    ],
  },
  {
    id: 'custom',
    name: 'custom',
    displayName: '自定义模型',
    description: '手动填写兼容 OpenAI Completions 的服务商信息与一个主模型。',
    baseUrl: '',
    isCustom: true,
    suggestedModels: [],
    providerModels: [],
  },
]

export const QUICK_SETUP_CHANNEL_PRESETS: QuickSetupChannelPreset[] = [
  {
    id: 'feishu',
    name: '飞书',
    description: '填写 App ID 和 App Secret。',
    placeholderLabel: 'App ID',
    secretLabel: 'App Secret',
  },
  {
    id: 'wecom',
    name: '企业微信',
    description: '填写 Bot ID 和 Secret。',
    placeholderLabel: 'Bot ID',
    secretLabel: 'Secret',
  },
  {
    id: 'qq',
    name: 'QQ',
    description: '填写 App ID 和 App Secret。',
    placeholderLabel: 'App ID',
    secretLabel: 'App Secret',
  },
  {
    id: 'dingtalk',
    name: '钉钉',
    description: '填写 Client ID 和 Client Secret。',
    placeholderLabel: 'Client ID',
    secretLabel: 'Client Secret',
  },
]

export const QUICK_SETUP_MANAGED_CHANNEL_IDS: QuickSetupChannelId[] = [...QUICK_SETUP_CHANNEL_ORDER]

export const canSkipQuickSetupStep = (stepId: QuickSetupStepId) =>
  stepId === 'model' || stepId === 'channel'

export const findProviderPreset = (presetId: string) =>
  QUICK_SETUP_PROVIDER_PRESETS.find((preset) => preset.id === presetId)

export const findChannelPreset = (channelId: string) =>
  QUICK_SETUP_CHANNEL_PRESETS.find((channel) => channel.id === channelId)

export const createQuickSetupCustomProviderPreset = ({
  providerName,
  baseUrl,
  selectedModelId,
}: QuickSetupCustomProviderInput): QuickSetupProviderPreset => ({
  id: 'custom',
  name: providerName.trim(),
  displayName: providerName.trim() || '自定义模型',
  description: '手动填写兼容 OpenAI Completions 的服务商信息与一个主模型。',
  baseUrl: baseUrl.trim(),
  isCustom: true,
  suggestedModels: selectedModelId.trim()
    ? [{ id: selectedModelId.trim(), name: selectedModelId.trim() }]
    : [],
  providerModels: selectedModelId.trim()
    ? [{ id: selectedModelId.trim(), name: selectedModelId.trim() }]
    : [],
})

export const buildQuickSetupModelOptions = ({
  fetchedModels,
  modelQuery,
}: QuickSetupModelOptionsInput): QuickSetupModelOption[] => {
  const unique = new Map<string, QuickSetupModelOption>()
  const typedModelId = modelQuery.trim()

  if (typedModelId) {
    unique.set(typedModelId, {
      id: typedModelId,
      name: typedModelId,
    })
  }

  for (const modelId of fetchedModels) {
    const trimmedModelId = modelId.trim()
    if (!trimmedModelId || unique.has(trimmedModelId)) continue
    unique.set(trimmedModelId, {
      id: trimmedModelId,
      name: trimmedModelId,
    })
  }

  return Array.from(unique.values())
}

export const applyQuickSetupModelPreset = (
  config: OpenClawConfig,
  preset: QuickSetupProviderPreset,
  apiKey: string,
  selectedModelId: string
): OpenClawConfig => {
  const next = cloneConfig(config)

  next.models ??= {}
  next.models.mode = 'merge'
  next.models.providers ??= {}
  next.models.providers[preset.name] = buildProviderConfig(preset, apiKey, selectedModelId)

  next.agents ??= {}
  next.agents.defaults ??= {}
  next.agents.defaults.model ??= { primary: '' }
  next.agents.defaults.model.primary = `${preset.name}/${selectedModelId}`

  const allowedModels = { ...(next.agents.defaults.models ?? {}) }
  for (const modelRef of Object.keys(allowedModels)) {
    if (modelRef.startsWith(`${preset.name}/`)) {
      delete allowedModels[modelRef]
    }
  }

  for (const model of next.models.providers[preset.name]?.models ?? []) {
    allowedModels[`${preset.name}/${model.id}`] = {}
  }
  allowedModels[`${preset.name}/${selectedModelId}`] = {}
  next.agents.defaults.models = allowedModels

  return next
}

export const clearQuickSetupManagedChannels = (config: OpenClawConfig): OpenClawConfig => {
  const next = cloneConfig(config)
  const channels = next.channels

  if (!channels || typeof channels !== 'object' || Array.isArray(channels)) {
    return next
  }

  const channelRecord = channels as Record<string, unknown>
  for (const channelId of QUICK_SETUP_MANAGED_CHANNEL_IDS) {
    delete channelRecord[channelId === 'qq' ? 'qqbot' : channelId]
  }
  delete channelRecord['dingtalk-connector']

  if (Object.keys(channelRecord).length === 0) {
    delete next.channels
  }

  return next
}

const hasNonEmptyString = (value: unknown) => typeof value === 'string' && value.trim().length > 0

const hasConfiguredQqToken = (node: Record<string, unknown>) =>
  hasNonEmptyString(node.token) || (hasNonEmptyString(node.appId) && hasNonEmptyString(node.clientSecret))

export const sanitizeQuickSetupChannelConfig = (config: OpenClawConfig): OpenClawConfig => {
  const next = cloneConfig(config)
  const channels = next.channels

  if (!channels || typeof channels !== 'object' || Array.isArray(channels)) {
    return next
  }

  const channelRecord = channels as Record<string, unknown>

  const feishu = channelRecord.feishu
  if (feishu && typeof feishu === 'object' && !Array.isArray(feishu)) {
    const node = feishu as Record<string, unknown>
    if (!hasNonEmptyString(node.appId) || !hasNonEmptyString(node.appSecret)) {
      delete channelRecord.feishu
    }
  }

  const wecom = channelRecord.wecom
  if (wecom && typeof wecom === 'object' && !Array.isArray(wecom)) {
    const node = wecom as Record<string, unknown>
    if (!hasNonEmptyString(node.botId) || !hasNonEmptyString(node.secret)) {
      delete channelRecord.wecom
    }
  }

  const qqbot = channelRecord.qqbot
  if (qqbot && typeof qqbot === 'object' && !Array.isArray(qqbot)) {
    const node = qqbot as Record<string, unknown>
    if (!hasConfiguredQqToken(node)) {
      delete channelRecord.qqbot
    }
  }

  const dingtalk = channelRecord.dingtalk
  if (dingtalk && typeof dingtalk === 'object' && !Array.isArray(dingtalk)) {
    const node = dingtalk as Record<string, unknown>
    if (!hasNonEmptyString(node.clientId) || !hasNonEmptyString(node.clientSecret)) {
      delete channelRecord.dingtalk
    }
  }

  const dingtalkConnector = channelRecord['dingtalk-connector']
  if (dingtalkConnector && typeof dingtalkConnector === 'object' && !Array.isArray(dingtalkConnector)) {
    const node = dingtalkConnector as Record<string, unknown>
    if (!hasNonEmptyString(node.clientId) || !hasNonEmptyString(node.clientSecret)) {
      delete channelRecord['dingtalk-connector']
    }
  }

  if (Object.keys(channelRecord).length === 0) {
    delete next.channels
  }

  return next
}

export const applyQuickSetupGatewayOptions = (
  config: OpenClawConfig,
  options: QuickSetupGatewayOptions
): OpenClawConfig => {
  const next = cloneConfig(config)

  const browserConfig = next.browser && typeof next.browser === 'object' && !Array.isArray(next.browser)
    ? { ...(next.browser as Record<string, unknown>) }
    : {}

  if (options.browserDefaultProfileEnabled) {
    browserConfig.defaultProfile = 'openclaw'
  } else {
    delete browserConfig.defaultProfile
  }

  if (Object.keys(browserConfig).length > 0) {
    next.browser = browserConfig
  } else {
    delete next.browser
  }

  const toolsConfig = next.tools && typeof next.tools === 'object' && !Array.isArray(next.tools)
    ? { ...(next.tools as Record<string, unknown>) }
    : {}

  if (options.toolsFullProfileEnabled) {
    toolsConfig.profile = 'full'
  } else {
    delete toolsConfig.profile
  }

  if (Object.keys(toolsConfig).length > 0) {
    next.tools = toolsConfig
  } else {
    delete next.tools
  }

  const hooksConfig =
    next.hooks && typeof next.hooks === 'object' && !Array.isArray(next.hooks)
      ? { ...(next.hooks as Record<string, unknown>) }
      : {}
  const internalHooks =
    hooksConfig.internal && typeof hooksConfig.internal === 'object' && !Array.isArray(hooksConfig.internal)
      ? { ...(hooksConfig.internal as Record<string, unknown>) }
      : {}
  const internalEntries =
    internalHooks.entries && typeof internalHooks.entries === 'object' && !Array.isArray(internalHooks.entries)
      ? { ...(internalHooks.entries as Record<string, unknown>) }
      : {}

  for (const entryId of QUICK_SETUP_INTERNAL_HOOK_ENTRY_IDS) {
    const existingEntry =
      internalEntries[entryId] && typeof internalEntries[entryId] === 'object' && !Array.isArray(internalEntries[entryId])
        ? { ...(internalEntries[entryId] as Record<string, unknown>) }
        : {}
    internalEntries[entryId] = {
      ...existingEntry,
      enabled: true,
    }
  }

  hooksConfig.internal = {
    ...internalHooks,
    enabled: true,
    entries: internalEntries,
  }
  next.hooks = hooksConfig

  return next
}

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
