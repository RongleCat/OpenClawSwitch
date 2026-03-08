export type JsonRecord = Record<string, unknown>

export const DINGTALK_CHANNEL_KEY = 'dingtalk'
export const DINGTALK_LEGACY_CHANNEL_KEY = 'dingtalk-connector'
export const DINGTALK_CHANNEL_BINDING_KEYS = [
  DINGTALK_CHANNEL_KEY,
  DINGTALK_LEGACY_CHANNEL_KEY,
]

const DINGTALK_EDITABLE_KEYS = [
  'enabled',
  'name',
  'clientId',
  'clientSecret',
  'robotCode',
  'corpId',
  'agentId',
  'dmPolicy',
  'groupPolicy',
  'allowFrom',
  'messageType',
  'cardTemplateId',
  'cardTemplateKey',
  'debug',
  'gatewayToken',
  'gatewayPassword',
  'sessionTimeout',
  'enableMediaUpload',
  'systemPrompt',
] as const

const asRecord = (value: unknown): JsonRecord | undefined =>
  value && typeof value === 'object' && !Array.isArray(value) ? (value as JsonRecord) : undefined

export const resolveDingtalkChannelNode = (channelsRaw: JsonRecord): JsonRecord =>
  asRecord(channelsRaw[DINGTALK_CHANNEL_KEY]) ||
  asRecord(channelsRaw[DINGTALK_LEGACY_CHANNEL_KEY]) ||
  {}

export const ensureDingtalkChannelConfigNode = (channelsRaw: JsonRecord): JsonRecord => {
  const current = asRecord(channelsRaw[DINGTALK_CHANNEL_KEY])
  const legacy = asRecord(channelsRaw[DINGTALK_LEGACY_CHANNEL_KEY])

  if (current && legacy) {
    const currentAccounts = asRecord(current.accounts)
    const legacyAccounts = asRecord(legacy.accounts)
    const merged: JsonRecord = {
      ...legacy,
      ...current,
    }

    if (legacyAccounts || currentAccounts) {
      merged.accounts = {
        ...(legacyAccounts || {}),
        ...(currentAccounts || {}),
      }
    }

    channelsRaw[DINGTALK_CHANNEL_KEY] = merged
    delete channelsRaw[DINGTALK_LEGACY_CHANNEL_KEY]
    return merged
  }

  if (current) {
    delete channelsRaw[DINGTALK_LEGACY_CHANNEL_KEY]
    return current
  }

  if (legacy) {
    channelsRaw[DINGTALK_CHANNEL_KEY] = legacy
    delete channelsRaw[DINGTALK_LEGACY_CHANNEL_KEY]
    return legacy
  }

  const created: JsonRecord = {}
  channelsRaw[DINGTALK_CHANNEL_KEY] = created
  return created
}

export const ensureDingtalkPluginAllowed = (root: JsonRecord) => {
  const plugins = asRecord(root.plugins) || {}
  root.plugins = plugins

  plugins.enabled = true

  const allow = Array.isArray(plugins.allow)
    ? plugins.allow.filter((item): item is string => typeof item === 'string')
    : []

  if (!allow.includes(DINGTALK_CHANNEL_KEY)) {
    allow.push(DINGTALK_CHANNEL_KEY)
  }

  plugins.allow = allow
}

export const shouldIncludeDingtalkDefaultAccount = ({
  channelNode,
  hasMultipleAgents,
}: {
  channelNode: JsonRecord
  hasMultipleAgents: boolean
}) => {
  if (hasMultipleAgents) {
    return true
  }

  const accounts = asRecord(channelNode.accounts)
  if (accounts && Object.keys(accounts).length > 0) {
    return true
  }

  return Object.keys(channelNode).some((key) => key !== 'accounts' && key !== 'defaultAccount')
}

export const mergeDingtalkEditableConfig = (
  existing: JsonRecord,
  next: JsonRecord
): JsonRecord => {
  const merged: JsonRecord = { ...existing }

  for (const key of DINGTALK_EDITABLE_KEYS) {
    delete merged[key]
  }

  return {
    ...merged,
    ...next,
  }
}
