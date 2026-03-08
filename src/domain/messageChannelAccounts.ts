import {
  DINGTALK_CHANNEL_KEY,
  ensureDingtalkChannelConfigNode,
  mergeDingtalkEditableConfig,
  resolveDingtalkChannelNode,
} from './dingtalkPlugin'

export type JsonRecord = Record<string, unknown>

export type ManagedMessageChannelId =
  | 'feishu'
  | 'dingtalk'
  | 'telegram'
  | 'discord'
  | 'slack'

export const MANAGED_MESSAGE_CHANNEL_IDS: ManagedMessageChannelId[] = [
  'feishu',
  'dingtalk',
  'telegram',
  'discord',
  'slack',
]

const ACCOUNT_SCOPED_KEYS: Record<ManagedMessageChannelId, string[]> = {
  telegram: ['enabled', 'botToken'],
  discord: ['enabled', 'token'],
  slack: ['enabled', 'botToken', 'appToken', 'signingSecret'],
  feishu: ['enabled', 'appId', 'appSecret'],
  dingtalk: ['enabled', 'name', 'clientId', 'clientSecret', 'robotCode', 'corpId', 'agentId'],
}

const SHARED_SCOPED_KEYS: Partial<Record<ManagedMessageChannelId, string[]>> = {
  dingtalk: [
    'dmPolicy',
    'groupPolicy',
    'allowFrom',
    'mediaUrlAllowlist',
    'showThinking',
    'debug',
    'messageType',
    'cardTemplateId',
    'cardTemplateKey',
    'groups',
    'maxConnectionAttempts',
    'initialReconnectDelay',
    'maxReconnectDelay',
    'reconnectJitter',
    'maxReconnectCycles',
    'useConnectionManager',
    'mediaMaxMb',
    'proactivePermissionHint',
  ],
}

const PUBLIC_SCOPED_KEYS: Record<ManagedMessageChannelId, string[]> = {
  telegram: [
    'dmPolicy',
    'allowFrom',
    'groupPolicy',
    'groupAllowFrom',
    'replyToMode',
    'defaultTo',
    'groups',
  ],
  discord: ['dm', 'groupPolicy', 'replyToMode', 'guilds'],
  slack: [
    'mode',
    'webhookPort',
    'webhookPath',
    'dmPolicy',
    'allowFrom',
    'groupPolicy',
    'channels',
    'replyToMode',
    'defaultTo',
    'requireMention',
    'textChunkLimit',
    'chunkMode',
  ],
  feishu: [
    'domain',
    'connectionMode',
    'dmPolicy',
    'allowFrom',
    'groupPolicy',
    'groupAllowFrom',
    'groupCommandMentionBypass',
    'webhookPort',
    'webhookPath',
    'encryptKey',
    'verificationToken',
    'renderMode',
    'mediaMaxMb',
    'dynamicAgentCreation',
  ],
  dingtalk: [...(SHARED_SCOPED_KEYS.dingtalk || [])],
}

const ACCOUNT_ID_PATTERN = /^[A-Za-z0-9][A-Za-z0-9._-]*$/

const asRecord = (value: unknown): JsonRecord | undefined =>
  value && typeof value === 'object' && !Array.isArray(value) ? (value as JsonRecord) : undefined

const asString = (value: unknown): string => {
  if (typeof value === 'string') return value.trim()
  if (typeof value === 'number') return String(value)
  return ''
}

const shortenCredential = (value: string): string => {
  const trimmed = value.trim()
  if (!trimmed) return ''
  if (trimmed.length <= 12) return trimmed
  return `${trimmed.slice(0, 6)}…${trimmed.slice(-4)}`
}

const hasCredentials = (channelId: ManagedMessageChannelId, node: JsonRecord | undefined): boolean => {
  if (!node) return false

  if (channelId === 'telegram') return asString(node.botToken).length > 0
  if (channelId === 'discord') return asString(node.token).length > 0
  if (channelId === 'slack') return asString(node.botToken).length > 0
  if (channelId === 'feishu') {
    return asString(node.appId).length > 0 && asString(node.appSecret).length > 0
  }

  return asString(node.clientId).length > 0 && asString(node.clientSecret).length > 0
}

const getCredentialHint = (channelId: ManagedMessageChannelId, node: JsonRecord | undefined): string => {
  if (!node) return ''

  if (channelId === 'telegram') return shortenCredential(asString(node.botToken))
  if (channelId === 'discord') return shortenCredential(asString(node.token))
  if (channelId === 'slack') return shortenCredential(asString(node.botToken))
  if (channelId === 'feishu') return asString(node.appId)

  return (
    shortenCredential(asString(node.clientId)) ||
    shortenCredential(asString(node.robotCode)) ||
    shortenCredential(asString(node.corpId))
  )
}

const getNameHint = (channelId: ManagedMessageChannelId, node: JsonRecord | undefined): string => {
  if (!node) return ''
  if (channelId === 'dingtalk') return asString(node.name)
  return ''
}

const getChannelKey = (channelId: ManagedMessageChannelId): string =>
  channelId === 'dingtalk' ? DINGTALK_CHANNEL_KEY : channelId

const sanitizeAccountNode = (
  channelId: ManagedMessageChannelId,
  accountNode: JsonRecord,
  existing?: JsonRecord
): JsonRecord => {
  const allowedKeys = new Set(ACCOUNT_SCOPED_KEYS[channelId].filter(key => key !== 'enabled'))
  const sharedKeys = new Set(SHARED_SCOPED_KEYS[channelId] || [])
  const next: JsonRecord = {}

  if (channelId === 'dingtalk' && existing) {
    for (const [key, value] of Object.entries(existing)) {
      if (!allowedKeys.has(key) && !sharedKeys.has(key)) {
        next[key] = value
      }
    }
  }

  for (const [key, value] of Object.entries(accountNode)) {
    if (allowedKeys.has(key)) {
      next[key] = value
    }
  }

  return next
}

const ensureRecord = (root: JsonRecord, key: string): JsonRecord => {
  const existing = asRecord(root[key])
  if (existing) return existing
  const next: JsonRecord = {}
  root[key] = next
  return next
}

export const getMessageChannelConfigNode = (
  channelsRaw: JsonRecord,
  channelId: ManagedMessageChannelId
): JsonRecord => {
  if (channelId === 'dingtalk') {
    return resolveDingtalkChannelNode(channelsRaw)
  }

  return asRecord(channelsRaw[channelId]) || {}
}

export const collectMessageChannelAccountIds = (
  channelNode: JsonRecord,
  extraAccountIds: string[] = []
): string[] => {
  const ids = new Set<string>(['default'])
  const accounts = asRecord(channelNode.accounts)

  if (accounts) {
    for (const key of Object.keys(accounts)) {
      const trimmed = key.trim()
      if (trimmed && trimmed !== 'default') ids.add(trimmed)
    }
  }

  for (const accountId of extraAccountIds) {
    const trimmed = accountId.trim()
    if (trimmed) ids.add(trimmed)
  }

  return Array.from(ids).sort((left, right) => {
    if (left === 'default') return -1
    if (right === 'default') return 1
    return left.localeCompare(right)
  })
}

export const buildMessageChannelAccountLabel = (
  channelId: ManagedMessageChannelId,
  accountId: string,
  accountNode: JsonRecord | undefined
): string => {
  const base = accountId === 'default' ? '默认账号' : accountId.trim() || '未命名账号'
  const nameHint = getNameHint(channelId, accountNode)
  const credentialHint = getCredentialHint(channelId, accountNode)
  const parts = [base]

  if (nameHint && nameHint !== base) {
    parts.push(nameHint)
  }

  if (credentialHint && credentialHint !== nameHint) {
    parts.push(credentialHint)
  }

  return parts.join(' · ')
}

export const isMessageChannelAccountIdValid = (value: string): boolean =>
  ACCOUNT_ID_PATTERN.test(value.trim())

export const isMessageChannelConfigured = (
  channelId: ManagedMessageChannelId,
  channelNode: JsonRecord
): boolean => {
  if (hasCredentials(channelId, channelNode)) {
    return true
  }

  const accounts = asRecord(channelNode.accounts)
  if (!accounts) return false

  return Object.values(accounts).some(account => hasCredentials(channelId, asRecord(account)))
}

export const extractMessageChannelPublicConfig = (
  channelId: ManagedMessageChannelId,
  channelNode: JsonRecord
): JsonRecord => {
  const next: JsonRecord = {}
  const accountKeys = new Set(ACCOUNT_SCOPED_KEYS[channelId])

  for (const [key, value] of Object.entries(channelNode)) {
    if (key === 'accounts' || key === 'defaultAccount') continue
    if (accountKeys.has(key)) continue
    next[key] = value
  }

  return next
}

export const saveMessageChannelDefaultAccountConfig = (
  root: JsonRecord,
  channelId: ManagedMessageChannelId,
  channelNode: JsonRecord
) => {
  const channels = ensureRecord(root, 'channels')
  const channelConfig =
    channelId === 'dingtalk'
      ? ensureDingtalkChannelConfigNode(channels)
      : ensureRecord(channels, getChannelKey(channelId))
  const accountKeys = ACCOUNT_SCOPED_KEYS[channelId]

  for (const key of accountKeys) {
    delete channelConfig[key]
  }

  for (const key of accountKeys) {
    if (!(key in channelNode)) continue
    channelConfig[key] = channelNode[key]
  }
}

export const saveMessageChannelAccountConfig = (
  root: JsonRecord,
  channelId: ManagedMessageChannelId,
  accountId: string,
  accountNode: JsonRecord
) => {
  const nextAccountId = accountId.trim()
  if (!nextAccountId || nextAccountId === 'default') return

  const channels = ensureRecord(root, 'channels')
  const channelConfig =
    channelId === 'dingtalk'
      ? ensureDingtalkChannelConfigNode(channels)
      : ensureRecord(channels, getChannelKey(channelId))
  const accounts = ensureRecord(channelConfig, 'accounts')
  const existingAccount = asRecord(accounts[nextAccountId])
  const sanitized = sanitizeAccountNode(channelId, accountNode, existingAccount)

  accounts[nextAccountId] =
    channelId === 'dingtalk'
      ? mergeDingtalkEditableConfig(existingAccount || {}, sanitized)
      : sanitized
}

export const saveMessageChannelPublicConfig = (
  root: JsonRecord,
  channelId: ManagedMessageChannelId,
  channelNode: JsonRecord
) => {
  const channels = ensureRecord(root, 'channels')
  const channelConfig =
    channelId === 'dingtalk'
      ? ensureDingtalkChannelConfigNode(channels)
      : ensureRecord(channels, getChannelKey(channelId))

  for (const key of PUBLIC_SCOPED_KEYS[channelId]) {
    delete channelConfig[key]
  }

  const publicConfig = extractMessageChannelPublicConfig(channelId, channelNode)
  for (const [key, value] of Object.entries(publicConfig)) {
    channelConfig[key] = value
  }
}

export const removeMessageChannelAccountConfig = (
  root: JsonRecord,
  channelId: ManagedMessageChannelId,
  accountId: string
) => {
  const nextAccountId = accountId.trim()
  if (!nextAccountId || nextAccountId === 'default') return

  const channels = asRecord(root.channels)
  if (!channels) return

  const channelConfig =
    channelId === 'dingtalk'
      ? asRecord(resolveDingtalkChannelNode(channels))
      : asRecord(channels[getChannelKey(channelId)])
  if (!channelConfig) return

  const accounts = asRecord(channelConfig.accounts)
  if (!accounts || !(nextAccountId in accounts)) return

  delete accounts[nextAccountId]

  if (Object.keys(accounts).length === 0) {
    delete channelConfig.accounts
  }

  if (asString(channelConfig.defaultAccount) === nextAccountId) {
    delete channelConfig.defaultAccount
  }
}
