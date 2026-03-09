export type JsonRecord = Record<string, unknown>

export const FEISHU_CHANNEL_KEY = 'feishu'
export const FEISHU_PLUGIN_ALLOW_ENTRY = 'feishu-openclaw-plugin'

export interface FeishuChannelConfigInput {
  enabled: boolean
  appId: string
  appSecret: string
  domain?: string
  connectionMode?: string
}

const asRecord = (value: unknown): JsonRecord | undefined =>
  value && typeof value === 'object' && !Array.isArray(value) ? (value as JsonRecord) : undefined

const ensureRecord = (root: JsonRecord, key: string): JsonRecord => {
  const existing = asRecord(root[key])
  if (existing) return existing
  const next: JsonRecord = {}
  root[key] = next
  return next
}

export const ensureFeishuPluginAllowed = (root: JsonRecord) => {
  const plugins = ensureRecord(root, 'plugins')
  plugins.enabled = true

  const allow = Array.isArray(plugins.allow)
    ? plugins.allow.filter((item): item is string => typeof item === 'string')
    : []

  if (!allow.includes(FEISHU_PLUGIN_ALLOW_ENTRY)) {
    allow.push(FEISHU_PLUGIN_ALLOW_ENTRY)
  }

  plugins.allow = allow
}

export const mergeFeishuChannelConfig = (
  root: JsonRecord,
  input: FeishuChannelConfigInput
) => {
  ensureFeishuPluginAllowed(root)

  const channels = ensureRecord(root, 'channels')
  const feishu = ensureRecord(channels, FEISHU_CHANNEL_KEY)

  feishu.enabled = input.enabled
  feishu.appId = input.appId.trim()
  feishu.appSecret = input.appSecret.trim()

  if (typeof input.domain === 'string') {
    feishu.domain = input.domain.trim()
  }

  if (typeof input.connectionMode === 'string') {
    feishu.connectionMode = input.connectionMode.trim()
  }
}
