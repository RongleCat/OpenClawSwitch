<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'
import { open as openExternal } from '@tauri-apps/api/shell'
import {
  Send,
  Hash,
  Slack,
  MessageCircle,
  Bell,
  ChevronDown,
  ChevronRight,
  Eye,
  EyeOff,
  Check,
  X,
  Save,
  KeyRound,
  ExternalLink,
  Download,
  Loader2,
  Plus,
  Trash2
} from 'lucide-vue-next'
import CommonInputConfirmModal from '../CommonInputConfirmModal.vue'
import Button from '../ui/Button.vue'
import Card from '../ui/Card.vue'
import Input from '../ui/Input.vue'
import TerminalLog from '../ui/TerminalLog.vue'
import {
  DINGTALK_CHANNEL_BINDING_KEYS,
  DINGTALK_CHANNEL_KEY,
  ensureDingtalkChannelConfigNode,
  ensureDingtalkPluginAllowed,
  mergeDingtalkEditableConfig,
  resolveDingtalkChannelNode,
} from '../../domain/dingtalkPlugin'
import {
  ensureFeishuPluginAllowed,
  mergeFeishuChannelConfig,
} from '../../domain/feishuPlugin'
import { messageChannelHeaderLayout } from '../../domain/messageChannelHeaderLayout'
import {
  MANAGED_MESSAGE_CHANNEL_IDS,
  buildMessageChannelAccountLabel,
  isMessageChannelAccountIdValid,
  isMessageChannelConfigured,
  removeMessageChannelAccountConfig,
  saveMessageChannelAccountConfig,
  saveMessageChannelDefaultAccountConfig,
  saveMessageChannelPublicConfig,
  type ManagedMessageChannelId,
} from '../../domain/messageChannelAccounts'
import {
  FEISHU_PAIRING_APPROVE_COMMAND_EXAMPLE,
  extractFeishuPairingCode,
} from '../../domain/feishuPairing'
import {
  canConfigureMessageChannelBeforeInstall,
  shouldBlockMessageChannelConfigUntilInstall,
} from '../../domain/messageChannelInstallGate'
import { messageChannelInstallModalLayout } from '../../domain/messageChannelInstallModalLayout'
import {
  getChannelConfigKey,
  isChannelPluginInstalled,
  MESSAGE_CHANNEL_PRIMARY_ORDER,
  PLUGIN_INSTALL_CHANNEL_IDS,
  sortMessageChannelsForDisplay,
  type ChannelPluginStatus,
} from '../../domain/channelPluginCatalog'
import {
  createDebouncedGatewayRestartController,
  resolveGatewayRestartCommand,
} from '../../domain/gatewayRestart'
import type { ConfigFileInfo, InstallLogEvent, OpenClawConfig } from '../../types/config'

type ChannelId =
  | 'wecom'
  | 'qq'
  | 'telegram'
  | 'discord'
  | 'slack'
  | 'feishu'
  | 'whatsapp'
  | 'imessage'
  | 'dingtalk'

type ChannelConfigPanel = 'credentials' | 'access' | 'connection' | 'advanced'
type ExtensionChannelId = 'feishu' | 'wecom' | 'qq' | 'dingtalk'
type JsonRecord = Record<string, unknown>

interface ChannelForm {
  token: string
  userId: string
  enabled: boolean

  privateStrategy: 'pair' | 'allow_all' | 'deny'
  groupStrategy: 'whitelist' | 'allow_all' | 'deny'

  telegramDmPolicy: 'pairing' | 'allowlist' | 'open' | 'disabled'
  telegramAllowFrom: string
  telegramGroupPolicy: 'allowlist' | 'open' | 'disabled'
  telegramGroupAllowFrom: string
  telegramReplyToMode: 'off' | 'first' | 'all'
  telegramDefaultTo: string
  telegramGroups: string

  discordDmPolicy: 'pairing' | 'allowlist' | 'open' | 'disabled'
  discordAllowFrom: string
  discordGroupPolicy: 'allowlist' | 'open' | 'disabled'
  discordGuildChannels: string
  discordReplyToMode: 'off' | 'first' | 'all'

  slackMode: 'http' | 'socket'
  slackBotToken: string
  slackAppToken: string
  slackSigningSecret: string
  slackWebhookPort: string
  slackWebhookPath: string
  slackDmPolicy: 'pairing' | 'allowlist' | 'open' | 'disabled'
  slackAllowFrom: string
  slackGroupPolicy: 'allowlist' | 'open' | 'disabled'
  slackChannels: string
  slackReplyToMode: 'off' | 'first' | 'all'
  slackDefaultTo: string
  slackRequireMention: boolean
  slackTextChunkLimit: string
  slackChunkMode: string

  whatsappSessionDir: string
  whatsappUseRemoteAuth: boolean
  whatsappWebhookPort: string
  whatsappWebhookPath: string
  whatsappDmPolicy: 'pairing' | 'allowlist' | 'open' | 'disabled'
  whatsappAllowFrom: string
  whatsappGroupPolicy: 'allowlist' | 'open' | 'disabled'
  whatsappGroupAllowFrom: string
  whatsappGroups: string
  whatsappIncludeAttachments: boolean
  whatsappMediaMaxMb: string
  whatsappTextChunkLimit: string
  whatsappChunkMode: string
  whatsappSendReadReceipts: boolean

  imessageCliPath: string
  imessageDbPath: string
  imessageRemoteHost: string
  imessageDmPolicy: 'pairing' | 'allowlist' | 'open' | 'disabled'
  imessageAllowFrom: string
  imessageGroupPolicy: 'allowlist' | 'open' | 'disabled'
  imessageGroupAllowFrom: string
  imessageGroups: string
  imessageIncludeAttachments: boolean
  imessageAttachmentRoots: string
  imessageRemoteAttachmentRoots: string
  imessageMediaMaxMb: string
  imessageService: string
  imessageRegion: string
  imessageTextChunkLimit: string
  imessageChunkMode: string

  feishuDomain: string
  feishuConnectionMode: 'websocket' | 'webhook'
  feishuDmPolicy: 'pairing' | 'allowlist' | 'open'
  feishuAllowFrom: string
  feishuGroupPolicy: 'allowlist' | 'open' | 'disabled'
  feishuGroupAllowFrom: string
  feishuGroupCommandMentionBypass: 'single_bot' | 'never' | 'always'
  feishuWebhookPort: string
  feishuWebhookPath: string
  feishuEncryptKey: string
  feishuVerificationToken: string
  feishuRenderMode: 'auto' | 'raw' | 'card'
  feishuMediaMaxMb: string
  feishuDynamicEnabled: boolean
  feishuDynamicWorkspaceTemplate: string
  feishuDynamicAgentDirTemplate: string
  feishuDynamicMaxAgents: string

  wecomName: string
  wecomDmPolicy: 'pairing' | 'allowlist' | 'open' | 'disabled'
  wecomAllowFrom: string
  wecomGroupPolicy: 'allowlist' | 'open' | 'disabled'
  wecomGroupAllowFrom: string
  wecomGroups: string
  wecomWebsocketUrl: string
  wecomSendThinkingMessage: boolean

  qqName: string
  qqDmPolicy: 'open' | 'pairing' | 'allowlist'
  qqAllowFrom: string
  qqSystemPrompt: string
  qqImageServerBaseUrl: string
  qqMarkdownSupport: boolean

  dingtalkDmPolicy: 'open' | 'pairing' | 'allowlist'
  dingtalkAllowFrom: string
  dingtalkGroupPolicy: 'open' | 'allowlist'
  dingtalkName: string
  dingtalkRobotCode: string
  dingtalkCorpId: string
  dingtalkAgentId: string
  dingtalkMessageType: 'markdown' | 'card'
  dingtalkCardTemplateId: string
  dingtalkCardTemplateKey: string
  dingtalkMediaUrlAllowlist: string
  dingtalkShowThinking: boolean
  dingtalkMediaMaxMb: string
  dingtalkDebug: boolean
}

interface ChannelMeta {
  id: ChannelId
  name: string
  icon: any
  iconColor: string
}

type ChannelExtensionStatus = ChannelPluginStatus

interface ChannelExtensionInstallStateEvent {
  channelId: string
  status: 'running' | 'success' | 'error'
}

const props = withDefaults(
  defineProps<{
    showToast: (type: 'success' | 'error', message: string) => void
    systemOs: 'windows' | 'macos' | 'linux'
  }>(),
  {}
)

const channelList: ChannelMeta[] = sortMessageChannelsForDisplay([
  { id: 'feishu', name: '飞书', icon: MessageCircle, iconColor: 'var(--oc-accent)' },
  { id: 'wecom', name: '企业微信', icon: MessageCircle, iconColor: 'var(--oc-success)' },
  { id: 'qq', name: 'QQ', icon: Bell, iconColor: 'var(--oc-warning)' },
  { id: 'dingtalk', name: '钉钉', icon: Bell, iconColor: 'var(--oc-accent)' },
  { id: 'telegram', name: 'Telegram', icon: Send, iconColor: 'var(--oc-accent)' },
  { id: 'discord', name: 'Discord', icon: Hash, iconColor: 'var(--oc-warning)' },
  { id: 'slack', name: 'Slack', icon: Slack, iconColor: 'var(--oc-warning)' },
])

const hints: Record<ChannelId, string> = {
  wecom: '按企业微信插件配置 Bot ID、Secret、访问策略、群组白名单与连接参数。',
  qq: '按 QQ Bot 插件配置 App ID、App Secret、私聊策略、系统提示词和 Markdown 能力。',
  telegram: '按官方配置填写 botToken、DM/群组策略、allowFrom 与 groups 白名单。',
  discord: '按官方配置填写 token、dm.policy、groupPolicy 与 guild/channel 白名单。',
  slack: '按官方配置填写 botToken、模式/端口/路径、访问策略、replyTo 与分块参数。',
  feishu: '按飞书插件 schema 配置 App 凭据、策略、连接模式与高级参数。',
  whatsapp: '按官方配置填写会话目录/远程认证、策略白名单与附件/分块参数。',
  imessage: '按官方配置填写 rustpush 路径、数据库路径、策略与附件/路由参数。',
  dingtalk: '按钉钉插件 schema 配置 Client 凭据、策略、网关与高级参数。'
}

const channelTabsMap: Record<ChannelId, Array<{ id: ChannelConfigPanel; label: string }>> = {
  telegram: [
    { id: 'credentials', label: '凭据配置' },
    { id: 'access', label: '访问策略' },
    { id: 'advanced', label: '高级配置' }
  ],
  discord: [
    { id: 'credentials', label: '凭据配置' },
    { id: 'access', label: '访问策略' },
    { id: 'advanced', label: '高级配置' }
  ],
  wecom: [
    { id: 'credentials', label: '凭据配置' },
    { id: 'access', label: '访问策略' },
    { id: 'connection', label: '连接模式' },
    { id: 'advanced', label: '高级配置' }
  ],
  qq: [
    { id: 'credentials', label: '凭据配置' },
    { id: 'access', label: '访问策略' },
    { id: 'advanced', label: '高级配置' }
  ],
  feishu: [
    { id: 'credentials', label: '凭据配置' },
    { id: 'access', label: '访问策略' },
    { id: 'connection', label: '连接模式' },
    { id: 'advanced', label: '高级配置' }
  ],
  dingtalk: [
    { id: 'credentials', label: '凭据配置' },
    { id: 'access', label: '访问策略' },
    { id: 'advanced', label: '高级配置' }
  ],
  slack: [
    { id: 'credentials', label: '凭据配置' },
    { id: 'access', label: '访问策略' },
    { id: 'connection', label: '连接模式' },
    { id: 'advanced', label: '高级配置' }
  ],
  whatsapp: [
    { id: 'credentials', label: '凭据配置' },
    { id: 'access', label: '访问策略' },
    { id: 'connection', label: '连接模式' },
    { id: 'advanced', label: '高级配置' }
  ],
  imessage: [
    { id: 'credentials', label: '凭据配置' },
    { id: 'access', label: '访问策略' },
    { id: 'connection', label: '连接模式' },
    { id: 'advanced', label: '高级配置' }
  ]
}

const storageKey = 'openclawswitch-message-channels'

const defaultForm = (): ChannelForm => ({
  token: '',
  userId: '',
  enabled: false,

  privateStrategy: 'pair',
  groupStrategy: 'whitelist',

  telegramDmPolicy: 'pairing',
  telegramAllowFrom: '',
  telegramGroupPolicy: 'allowlist',
  telegramGroupAllowFrom: '',
  telegramReplyToMode: 'off',
  telegramDefaultTo: '',
  telegramGroups: '',

  discordDmPolicy: 'pairing',
  discordAllowFrom: '',
  discordGroupPolicy: 'allowlist',
  discordGuildChannels: '',
  discordReplyToMode: 'off',

  slackMode: 'http',
  slackBotToken: '',
  slackAppToken: '',
  slackSigningSecret: '',
  slackWebhookPort: '',
  slackWebhookPath: '/webhooks/slack',
  slackDmPolicy: 'pairing',
  slackAllowFrom: '',
  slackGroupPolicy: 'allowlist',
  slackChannels: '',
  slackReplyToMode: 'off',
  slackDefaultTo: '',
  slackRequireMention: true,
  slackTextChunkLimit: '',
  slackChunkMode: 'sentence',

  whatsappSessionDir: '',
  whatsappUseRemoteAuth: false,
  whatsappWebhookPort: '',
  whatsappWebhookPath: '/webhooks/whatsapp',
  whatsappDmPolicy: 'pairing',
  whatsappAllowFrom: '',
  whatsappGroupPolicy: 'allowlist',
  whatsappGroupAllowFrom: '',
  whatsappGroups: '',
  whatsappIncludeAttachments: true,
  whatsappMediaMaxMb: '',
  whatsappTextChunkLimit: '',
  whatsappChunkMode: 'sentence',
  whatsappSendReadReceipts: false,

  imessageCliPath: 'rustpush',
  imessageDbPath: '',
  imessageRemoteHost: '',
  imessageDmPolicy: 'pairing',
  imessageAllowFrom: '',
  imessageGroupPolicy: 'allowlist',
  imessageGroupAllowFrom: '',
  imessageGroups: '',
  imessageIncludeAttachments: true,
  imessageAttachmentRoots: '',
  imessageRemoteAttachmentRoots: '',
  imessageMediaMaxMb: '',
  imessageService: 'auto',
  imessageRegion: 'CN',
  imessageTextChunkLimit: '',
  imessageChunkMode: 'sentence',

  feishuDomain: 'feishu',
  feishuConnectionMode: 'websocket',
  feishuDmPolicy: 'pairing',
  feishuAllowFrom: '',
  feishuGroupPolicy: 'allowlist',
  feishuGroupAllowFrom: '',
  feishuGroupCommandMentionBypass: 'single_bot',
  feishuWebhookPort: '',
  feishuWebhookPath: '/feishu/events',
  feishuEncryptKey: '',
  feishuVerificationToken: '',
  feishuRenderMode: 'auto',
  feishuMediaMaxMb: '',
  feishuDynamicEnabled: false,
  feishuDynamicWorkspaceTemplate: '',
  feishuDynamicAgentDirTemplate: '',
  feishuDynamicMaxAgents: '',

  wecomName: '',
  wecomDmPolicy: 'pairing',
  wecomAllowFrom: '',
  wecomGroupPolicy: 'open',
  wecomGroupAllowFrom: '',
  wecomGroups: '',
  wecomWebsocketUrl: '',
  wecomSendThinkingMessage: true,

  qqName: '',
  qqDmPolicy: 'pairing',
  qqAllowFrom: '',
  qqSystemPrompt: '',
  qqImageServerBaseUrl: '',
  qqMarkdownSupport: true,

  dingtalkDmPolicy: 'open',
  dingtalkAllowFrom: '',
  dingtalkGroupPolicy: 'open',
  dingtalkName: '',
  dingtalkRobotCode: '',
  dingtalkCorpId: '',
  dingtalkAgentId: '',
  dingtalkMessageType: 'markdown',
  dingtalkCardTemplateId: '',
  dingtalkCardTemplateKey: 'content',
  dingtalkMediaUrlAllowlist: '',
  dingtalkShowThinking: true,
  dingtalkMediaMaxMb: '',
  dingtalkDebug: false
})

const parseQqCredentialsFromConfig = (node: JsonRecord | undefined) => {
  if (!node) return { appId: '', clientSecret: '' }

  const appId = asString(node.appId)
  const clientSecret = asString(node.clientSecret)
  if (appId || clientSecret) {
    return { appId, clientSecret }
  }

  const token = asString(node.token)
  if (!token) return { appId: '', clientSecret: '' }

  const [legacyAppId, ...secretParts] = token.split(':')
  return {
    appId: legacyAppId?.trim() || '',
    clientSecret: secretParts.join(':').trim(),
  }
}

const channelIds: ChannelId[] = [
  ...MESSAGE_CHANNEL_PRIMARY_ORDER,
  'telegram',
  'discord',
  'slack',
  'whatsapp',
  'imessage',
]

interface AgentOption {
  id: string
  label: string
}

interface AccountOption {
  id: string
  label: string
}

const buildChannelRecord = <T>(factory: (channelId: ChannelId) => T): Record<ChannelId, T> =>
  channelIds.reduce((acc, id) => {
    acc[id] = factory(id)
    return acc
  }, {} as Record<ChannelId, T>)

const loadStoredForms = (): Record<ChannelId, ChannelForm> => {
  const initial = channelIds.reduce((acc, id) => {
    acc[id] = defaultForm()
    return acc
  }, {} as Record<ChannelId, ChannelForm>)

  try {
    const raw = localStorage.getItem(storageKey)
    if (!raw) return initial

    const parsed = JSON.parse(raw) as Partial<Record<ChannelId, Partial<ChannelForm>>>
    for (const id of channelIds) {
      const stored = parsed[id]
      if (!stored || typeof stored !== 'object') continue
      initial[id] = {
        ...initial[id],
        ...stored
      }
    }
  } catch {
    return initial
  }

  return initial
}

const forms = ref<Record<ChannelId, ChannelForm>>(loadStoredForms())
const availableAgents = ref<AgentOption[]>([{ id: 'default', label: 'default' }])
const selectedAgentByChannel = ref<Record<ChannelId, string>>(buildChannelRecord(() => 'default'))
const configuredByChannel = ref<Record<ChannelId, boolean>>(buildChannelRecord(() => false))
const availableAccountsByChannel = ref<Record<ChannelId, AccountOption[]>>(
  buildChannelRecord(() => [{ id: 'default', label: '默认账号' }])
)
const selectedAccountByChannel = ref<Record<ChannelId, string>>(buildChannelRecord(() => 'default'))
const selectedChannelId = ref<ChannelId>(channelList[0].id)
const revealToken = ref(false)
const revealSecret = ref(false)
const revealSlackSigningSecret = ref(false)
const selectedPanel = ref<ChannelConfigPanel>(channelTabsMap[selectedChannelId.value][0].id)
const extensionStatus = ref<ChannelExtensionStatus>({
  feishuInstalled: false,
  wecomInstalled: false,
  qqInstalled: false,
  dingtalkInstalled: false
})
const gatewayRestartController = createDebouncedGatewayRestartController()
const extensionStatusLoading = ref(false)
const showInstallModal = ref(false)
const installingExtension = ref(false)
const installLogs = ref<InstallLogEvent[]>([])
const installingChannel = ref<ExtensionChannelId | null>(null)
const showPairingCodeModal = ref(false)
const pairingInput = ref('')
const approvingPairing = ref(false)
const showAccountModal = ref(false)
const accountInput = ref('')
const submittingAccount = ref(false)
const showAccountSelectorDropdown = ref(false)

let unlistenExtensionInstallLog: (() => void) | null = null
let unlistenExtensionInstallState: (() => void) | null = null

const isRecord = (value: unknown): value is JsonRecord =>
  typeof value === 'object' && value !== null && !Array.isArray(value)

const asRecord = (value: unknown): JsonRecord | undefined => (isRecord(value) ? value : undefined)

const ensureRecord = (root: JsonRecord, key: string): JsonRecord => {
  const existing = asRecord(root[key])
  if (existing) return existing
  const next: JsonRecord = {}
  root[key] = next
  return next
}

const asString = (value: unknown): string => {
  if (typeof value === 'string') return value
  if (typeof value === 'number') return String(value)
  return ''
}

const parseListText = (raw: string): string[] =>
  raw
    .split(/\r?\n|,/)
    .map(entry => entry.trim())
    .filter(Boolean)

const listToText = (value: unknown): string => {
  if (!Array.isArray(value)) return ''
  return value
    .map(entry => String(entry).trim())
    .filter(Boolean)
    .join('\n')
}

const parsePositiveInt = (raw: string): number | undefined => {
  const value = raw.trim()
  if (!value) return undefined
  const number = Number(value)
  if (!Number.isInteger(number) || number <= 0) return undefined
  return number
}

const parsePositiveNumber = (raw: string): number | undefined => {
  const value = raw.trim()
  if (!value) return undefined
  const number = Number(value)
  if (!Number.isFinite(number) || number <= 0) return undefined
  return number
}

const enumOrDefault = <T extends string>(
  value: unknown,
  options: readonly T[],
  fallback: T
): T => {
  if (typeof value !== 'string') return fallback
  return options.includes(value as T) ? (value as T) : fallback
}

const getPathValue = (root: JsonRecord, path: string[]): unknown => {
  let current: unknown = root
  for (const part of path) {
    if (!isRecord(current)) return undefined
    current = current[part]
  }
  return current
}

const ensurePathRecord = (root: JsonRecord, path: string[]): JsonRecord => {
  let current = root
  for (const part of path) {
    const next = current[part]
    if (!isRecord(next)) {
      current[part] = {}
    }
    current = current[part] as JsonRecord
  }
  return current
}

const setPathValue = (root: JsonRecord, path: string[], value: unknown) => {
  const parent = ensurePathRecord(root, path.slice(0, -1))
  parent[path[path.length - 1]] = value
}

const deletePathValue = (root: JsonRecord, path: string[]) => {
  let current: unknown = root
  for (let i = 0; i < path.length - 1; i += 1) {
    if (!isRecord(current)) return
    current = current[path[i]]
  }
  if (!isRecord(current)) return
  delete current[path[path.length - 1]]
}

const setStringOrDelete = (root: JsonRecord, path: string[], value: string) => {
  const trimmed = value.trim()
  if (trimmed) {
    setPathValue(root, path, trimmed)
  } else {
    deletePathValue(root, path)
  }
}

const setListOrDelete = (root: JsonRecord, path: string[], value: string) => {
  const entries = parseListText(value)
  if (entries.length > 0) {
    setPathValue(root, path, entries)
  } else {
    deletePathValue(root, path)
  }
}

const setNumberOrDelete = (
  root: JsonRecord,
  path: string[],
  value: string,
  parser: (raw: string) => number | undefined
) => {
  const parsed = parser(value)
  if (typeof parsed === 'number') {
    setPathValue(root, path, parsed)
  } else {
    deletePathValue(root, path)
  }
}

interface DiscordGuildChannelEntry {
  guildId: string
  channelId?: string
}

const parseDiscordGuildChannels = (raw: string): DiscordGuildChannelEntry[] => {
  const lines = parseListText(raw)
  const entries: DiscordGuildChannelEntry[] = []

  for (const line of lines) {
    const slashIndex = line.indexOf('/')
    if (slashIndex === -1) {
      entries.push({ guildId: line })
      continue
    }

    const guildId = line.slice(0, slashIndex).trim()
    const channelId = line.slice(slashIndex + 1).trim()
    if (!guildId) continue
    if (!channelId) {
      entries.push({ guildId })
      continue
    }
    entries.push({ guildId, channelId })
  }

  return entries
}

const buildDiscordGuildChannelsText = (value: unknown): string => {
  const guilds = asRecord(value)
  if (!guilds) return ''

  const lines: string[] = []
  for (const guildId of Object.keys(guilds)) {
    const guild = asRecord(guilds[guildId])
    const channels = asRecord(guild?.channels)
    const channelIds = channels ? Object.keys(channels) : []

    if (channelIds.length === 0) {
      lines.push(guildId)
      continue
    }

    for (const channelId of channelIds) {
      lines.push(`${guildId}/${channelId}`)
    }
  }

  return lines.join('\n')
}

const buildObjectKeysText = (value: unknown): string => {
  const obj = asRecord(value)
  if (!obj) return ''
  return Object.keys(obj).join('\n')
}

const channelConfigKey = (channelId: ChannelId): string =>
  channelId === 'dingtalk' ? DINGTALK_CHANNEL_KEY : getChannelConfigKey(channelId)

const channelBindingKeys = (channelId: ChannelId): string[] => {
  if (channelId === 'dingtalk') return DINGTALK_CHANNEL_BINDING_KEYS
  const configKey = getChannelConfigKey(channelId)
  return configKey === channelId ? [channelId] : [channelId, configKey]
}

const bindingChannelMatches = (channelId: ChannelId, rawChannel: unknown): boolean => {
  if (typeof rawChannel !== 'string') return false
  return channelBindingKeys(channelId).includes(rawChannel)
}

const getChannelConfigNode = (channelsRaw: JsonRecord, channelId: ChannelId): JsonRecord => {
  if (channelId === 'dingtalk') {
    return resolveDingtalkChannelNode(channelsRaw)
  }
  const key = channelConfigKey(channelId)
  return asRecord(channelsRaw[key]) || {}
}

const getAccountNode = (channelNode: JsonRecord, accountId: string): JsonRecord | undefined => {
  const accounts = asRecord(channelNode.accounts)
  if (!accounts) return undefined
  return asRecord(accounts[accountId])
}

const readChannelValue = (channelNode: JsonRecord, accountId: string, path: string[]): unknown => {
  const accountNode = getAccountNode(channelNode, accountId)
  if (accountNode) {
    const accountValue = getPathValue(accountNode, path)
    if (accountValue !== undefined) return accountValue
  }

  const sharedValue = getPathValue(channelNode, path)
  if (sharedValue !== undefined) return sharedValue

  if (accountId !== 'default') {
    // Backward compatibility: some legacy configs store shared fields under accounts.default.
    const defaultAccountNode = getAccountNode(channelNode, 'default')
    if (defaultAccountNode) {
      const defaultAccountValue = getPathValue(defaultAccountNode, path)
      if (defaultAccountValue !== undefined) return defaultAccountValue
    }
  }
  return undefined
}

const readSharedChannelValue = (channelNode: JsonRecord, path: string[]): unknown => {
  const sharedValue = getPathValue(channelNode, path)
  if (sharedValue !== undefined) return sharedValue

  const defaultAccountValue = getPathValue(getAccountNode(channelNode, 'default') || {}, path)
  if (defaultAccountValue !== undefined) return defaultAccountValue

  const accounts = asRecord(channelNode.accounts)
  if (!accounts) return undefined

  for (const accountId of Object.keys(accounts)) {
    const accountValue = getPathValue(asRecord(accounts[accountId]) || {}, path)
    if (accountValue !== undefined) return accountValue
  }

  return undefined
}

const parseAgentOptions = (root: JsonRecord): AgentOption[] => {
  void root
  return [{ id: 'default', label: 'default' }]
}

const parseBindings = (root: JsonRecord): JsonRecord[] => {
  void root
  return []
}

const resolveBindingAccountId = (
  bindings: JsonRecord[],
  channelId: ChannelId,
  agentId: string
): string | undefined => {
  for (const binding of bindings) {
    if (asString(binding.agentId).trim() !== agentId) continue
    const match = asRecord(binding.match)
    if (!match) continue
    if (!bindingChannelMatches(channelId, match.channel)) continue
    const accountId = asString(match.accountId).trim()
    if (accountId) return accountId
  }
  return undefined
}

const collectChannelAccountIds = (
  channelNode: JsonRecord,
  bindings: JsonRecord[],
  channelId: ChannelId,
  agentOptions: AgentOption[]
): string[] => {
  const ids = new Set<string>()

  const accounts = asRecord(channelNode.accounts)
  if (accounts) {
    for (const key of Object.keys(accounts)) {
      if (key.trim()) ids.add(key.trim())
    }
  }

  for (const binding of bindings) {
    const match = asRecord(binding.match)
    if (!match) continue
    if (!bindingChannelMatches(channelId, match.channel)) continue
    const accountId = asString(match.accountId).trim()
    if (accountId) ids.add(accountId)
  }

  if (agentOptions.length > 1) {
    for (const agent of agentOptions) {
      const agentId = agent.id.trim()
      if (agentId) ids.add(agentId)
    }
  }

  if (ids.size === 0) ids.add('default')
  const sorted = Array.from(ids)
  sorted.sort((a, b) => {
    if (a === 'default') return -1
    if (b === 'default') return 1
    return a.localeCompare(b)
  })
  return sorted
}

const resolveChannelDefaultAccountId = (channelNode: JsonRecord, accountIds: string[]): string => {
  const configuredDefaultAccount = asString(channelNode.defaultAccount).trim()
  if (configuredDefaultAccount && accountIds.includes(configuredDefaultAccount)) {
    return configuredDefaultAccount
  }
  if (accountIds.includes('default')) return 'default'
  return accountIds[0] || 'default'
}

const getSelectedAgentId = (channelId: ChannelId): string => {
  const current = selectedAgentByChannel.value[channelId]
  if (current && availableAgents.value.some(agent => agent.id === current)) return current
  return availableAgents.value[0]?.id || 'default'
}

const getSelectedAccountId = (channelId: ChannelId): string => {
  const options = availableAccountsByChannel.value[channelId] || [{ id: 'default', label: '默认账号' }]
  const current = selectedAccountByChannel.value[channelId]
  if (current && options.some(option => option.id === current)) return current
  if (options.some(option => option.id === 'default')) return 'default'
  return options[0]?.id || 'default'
}

const isAccountModeEnabled = (channelId: ChannelId): boolean => {
  const accounts = availableAccountsByChannel.value[channelId] || []
  return accounts.some(account => account.id !== 'default')
}

const deriveChannelEnabledFromAccounts = (channelNode: JsonRecord): boolean | undefined => {
  let hasExplicitAccountEnabled = typeof channelNode.enabled === 'boolean'
  let anyEnabled = typeof channelNode.enabled === 'boolean' ? Boolean(channelNode.enabled) : false
  const accounts = asRecord(channelNode.accounts)
  if (!accounts) {
    return hasExplicitAccountEnabled ? anyEnabled : undefined
  }

  for (const accountId of Object.keys(accounts)) {
    const account = asRecord(accounts[accountId])
    if (!account) continue
    if (typeof account.enabled !== 'boolean') continue
    hasExplicitAccountEnabled = true
    if (account.enabled) {
      anyEnabled = true
      break
    }
  }

  if (!hasExplicitAccountEnabled) return undefined
  return anyEnabled
}

const syncChannelEnabledFromAccounts = (
  mutable: JsonRecord,
  channelId: ChannelId,
  fallbackEnabled: boolean
) => {
  const key = channelConfigKey(channelId)
  const channels = asRecord(mutable.channels) || {}
  const channelNode =
    channelId === 'dingtalk'
      ? resolveDingtalkChannelNode(channels)
      : asRecord(getPathValue(mutable, ['channels', key])) || {}
  const derivedEnabled = deriveChannelEnabledFromAccounts(channelNode)
  setPathValue(
    mutable,
    ['channels', key, 'enabled'],
    typeof derivedEnabled === 'boolean' ? derivedEnabled : fallbackEnabled
  )
}

const applyChannelConfigToAccount = (
  mutable: JsonRecord,
  channelId: ChannelId,
  accountId: string,
  form: ChannelForm,
  applyChannelConfig: (draft: JsonRecord, form: ChannelForm) => void
) => {
  const draft: JsonRecord = {}
  applyChannelConfig(draft, form)

  const key = channelConfigKey(channelId)
  const channelConfig = asRecord(getPathValue(draft, ['channels', key]))
  if (!channelConfig) return

  const nextAccountId = accountId.trim() || 'default'
  if (
    channelId === 'telegram' ||
    channelId === 'discord' ||
    channelId === 'slack' ||
    channelId === 'feishu' ||
    channelId === 'wecom' ||
    channelId === 'qq' ||
    channelId === 'dingtalk'
  ) {
    saveMessageChannelAccountConfig(
      mutable,
      channelId as ManagedMessageChannelId,
      nextAccountId,
      channelConfig
    )
  } else {
    setPathValue(mutable, ['channels', key, 'accounts', nextAccountId], channelConfig)
  }
  syncChannelEnabledFromAccounts(mutable, channelId, form.enabled)
}

const buildManagedChannelDraftNode = (
  channelId: ManagedMessageChannelId,
  form: ChannelForm,
  applyChannelConfig: (draft: JsonRecord, form: ChannelForm) => void
): JsonRecord => {
  const draft: JsonRecord = {}
  applyChannelConfig(draft, form)
  return asRecord(getPathValue(draft, ['channels', channelConfigKey(channelId)])) || {}
}

const upsertAgentBinding = (
  mutable: JsonRecord,
  channelId: ChannelId,
  agentId: string,
  accountId: string
) => {
  void mutable
  void channelId
  void agentId
  void accountId
}

watch(
  forms,
  value => {
    localStorage.setItem(storageKey, JSON.stringify(value))
  },
  { deep: true }
)

watch(selectedChannelId, () => {
  const tabs = channelTabsMap[selectedChannelId.value]
  selectedPanel.value = tabs[0]?.id ?? 'credentials'
  revealToken.value = false
  revealSecret.value = false
  revealSlackSigningSecret.value = false
  showAccountSelectorDropdown.value = false
  void syncChannelsFromConfig()
})

const selectedChannel = computed(() =>
  channelList.find(channel => channel.id === selectedChannelId.value) || channelList[0]
)
const panelTabs = computed(() => channelTabsMap[selectedChannelId.value])
const installingChannelName = computed(() =>
  channelList.find(channel => channel.id === installingChannel.value)?.name || ''
)

const currentForm = computed(() => forms.value[selectedChannelId.value])

const currentAccountOptions = computed(
  () => availableAccountsByChannel.value[selectedChannelId.value] || [{ id: 'default', label: '默认账号' }]
)
const showAccountSelector = computed(() => currentAccountOptions.value.length > 0)
const currentAccountId = computed(() => getSelectedAccountId(selectedChannelId.value))
const isManagedChannel = (channelId: ChannelId): channelId is ManagedMessageChannelId =>
  MANAGED_MESSAGE_CHANNEL_IDS.includes(channelId as ManagedMessageChannelId)
const publicPanelTabs = computed(() =>
  panelTabs.value
    .filter(tab => tab.id !== 'credentials')
    .map(tab => ({
      ...tab,
      label: `公共${tab.label}`,
    }))
)
const currentAccountTabLabel = computed(() => {
  const accountLabel = currentAccountId.value === 'default' ? '默认账号' : currentAccountId.value
  return `账号独立配置 · ${accountLabel}`
})
const currentChannelConfigRootPath = computed(
  () => `channels.${channelConfigKey(selectedChannelId.value)}`
)
const currentAccountConfigPath = computed(() =>
  currentAccountId.value === 'default'
    ? currentChannelConfigRootPath.value
    : `${currentChannelConfigRootPath.value}.accounts.${currentAccountId.value}`
)
const currentPanelScopeHint = computed(() =>
  selectedPanel.value === 'credentials'
    ? `当前为账号独立配置，保存到 ${currentAccountConfigPath.value}。`
    : `当前为公共配置，影响当前渠道所有账号，保存到 ${currentChannelConfigRootPath.value}。`
)
const saveButtonLabel = computed(() =>
  selectedPanel.value === 'credentials' ? '保存账号配置' : '保存公共配置'
)
const accountScopedFieldPath = (field: string) => `${currentAccountConfigPath.value}.${field}`

const toggleAccountSelectorDropdown = () => {
  if (!showAccountSelector.value || !canConfigureCurrentChannel.value) return
  selectedPanel.value = 'credentials'
  showAccountSelectorDropdown.value = !showAccountSelectorDropdown.value
}

const handleAccountSelectionChange = async (value: string) => {
  const channelId = selectedChannelId.value
  const next = value.trim()
  if (!next) return
  selectedPanel.value = 'credentials'
  selectedAccountByChannel.value[channelId] = next
  showAccountSelectorDropdown.value = false
  await syncChannelsFromConfig()
}

const handleAccountInputChange = (value: string) => {
  accountInput.value = value
}

const openAccountModal = () => {
  accountInput.value = ''
  selectedPanel.value = 'credentials'
  showAccountSelectorDropdown.value = false
  showAccountModal.value = true
}

const closeAccountModal = () => {
  if (submittingAccount.value) return
  showAccountModal.value = false
}

const submitAccount = async () => {
  const channelId = selectedChannelId.value
  const nextAccountId = accountInput.value.trim()
  if (!nextAccountId) return

  if (!isMessageChannelAccountIdValid(nextAccountId)) {
    props.showToast('error', '账号 ID 仅支持字母、数字、点、下划线和中划线')
    return
  }

  if (availableAccountsByChannel.value[channelId]?.some(option => option.id === nextAccountId)) {
    props.showToast('error', `账号 ${nextAccountId} 已存在`)
    return
  }

  const applyChannelConfig =
    channelId === 'telegram'
      ? applyTelegramConfig
      : channelId === 'discord'
        ? applyDiscordConfig
        : channelId === 'slack'
          ? applySlackConfig
          : channelId === 'wecom'
            ? applyWecomConfig
            : channelId === 'qq'
              ? applyQqConfig
          : channelId === 'feishu'
            ? applyFeishuConfig
            : applyDingtalkConfig

  submittingAccount.value = true
  try {
    await persistConfigMutation(mutable => {
      if (channelId === 'dingtalk') {
        ensureDingtalkPluginAllowed(mutable)
      }
      applyChannelConfigToAccount(mutable, channelId, nextAccountId, currentForm.value, applyChannelConfig)
    })
    selectedAccountByChannel.value[channelId] = nextAccountId
    showAccountModal.value = false
    accountInput.value = ''
    await syncChannelsFromConfig()
    props.showToast('success', `已新增账号 ${nextAccountId}`)
  } catch (error) {
    props.showToast('error', String(error))
  } finally {
    submittingAccount.value = false
  }
}

const removeSelectedAccount = async () => {
  const channelId = selectedChannelId.value
  const accountId = getSelectedAccountId(channelId)
  if (accountId === 'default') return
  if (!window.confirm(`确认删除账号 ${accountId}？`)) return

  try {
    await persistConfigMutation(mutable => {
      if (
        channelId === 'telegram' ||
        channelId === 'discord' ||
        channelId === 'slack' ||
        channelId === 'feishu' ||
        channelId === 'wecom' ||
        channelId === 'qq' ||
        channelId === 'dingtalk'
      ) {
        removeMessageChannelAccountConfig(mutable, channelId as ManagedMessageChannelId, accountId)
      }
    })
    selectedAccountByChannel.value[channelId] = 'default'
    showAccountSelectorDropdown.value = false
    await syncChannelsFromConfig()
    props.showToast('success', `已删除账号 ${accountId}`)
  } catch (error) {
    props.showToast('error', String(error))
  }
}

const isConfigured = (id: ChannelId) => {
  return Boolean(configuredByChannel.value[id])
}

const isExtensionChannel = (id: ChannelId): id is ExtensionChannelId =>
  PLUGIN_INSTALL_CHANNEL_IDS.includes(id as ExtensionChannelId)

const selectedExtensionInstalled = computed(() =>
  isChannelPluginInstalled(extensionStatus.value, selectedChannelId.value)
)

const selectedNeedsExtensionInstall = computed(() =>
  shouldBlockMessageChannelConfigUntilInstall({
    channelId: selectedChannelId.value,
    isExtensionChannel: isExtensionChannel(selectedChannelId.value),
    installed: selectedExtensionInstalled.value,
  })
)

const canConfigureCurrentChannel = computed(() => {
  if (!isExtensionChannel(selectedChannelId.value)) return true

  return (
    selectedExtensionInstalled.value ||
    canConfigureMessageChannelBeforeInstall(selectedChannelId.value)
  )
})
const enabledLabel = computed(() => (currentForm.value.enabled ? '已启用' : '已停用'))

const loadLocalConfig = async () => invoke<[OpenClawConfig, ConfigFileInfo]>('load_default_config')

const applyTelegramConfig = (mutable: JsonRecord, form: ChannelForm) => {
  setPathValue(mutable, ['channels', 'telegram', 'enabled'], form.enabled)
  setStringOrDelete(mutable, ['channels', 'telegram', 'botToken'], form.token)
  setPathValue(mutable, ['channels', 'telegram', 'dmPolicy'], form.telegramDmPolicy)
  setListOrDelete(mutable, ['channels', 'telegram', 'allowFrom'], form.telegramAllowFrom)
  setPathValue(mutable, ['channels', 'telegram', 'groupPolicy'], form.telegramGroupPolicy)
  setListOrDelete(mutable, ['channels', 'telegram', 'groupAllowFrom'], form.telegramGroupAllowFrom)
  setPathValue(mutable, ['channels', 'telegram', 'replyToMode'], form.telegramReplyToMode)
  setStringOrDelete(mutable, ['channels', 'telegram', 'defaultTo'], form.telegramDefaultTo)

  const existingGroups = asRecord(getPathValue(mutable, ['channels', 'telegram', 'groups'])) || {}
  const groupIds = parseListText(form.telegramGroups)
  if (groupIds.length === 0) {
    deletePathValue(mutable, ['channels', 'telegram', 'groups'])
  } else {
    const groups: JsonRecord = {}
    for (const groupId of groupIds) {
      const existing = asRecord(existingGroups[groupId])
      groups[groupId] = existing ? { ...existing } : { enabled: true }
    }
    setPathValue(mutable, ['channels', 'telegram', 'groups'], groups)
  }
}

const applyDiscordConfig = (mutable: JsonRecord, form: ChannelForm) => {
  setPathValue(mutable, ['channels', 'discord', 'enabled'], form.enabled)
  setStringOrDelete(mutable, ['channels', 'discord', 'token'], form.token)
  setPathValue(mutable, ['channels', 'discord', 'dm', 'policy'], form.discordDmPolicy)
  setListOrDelete(mutable, ['channels', 'discord', 'dm', 'allowFrom'], form.discordAllowFrom)
  setPathValue(mutable, ['channels', 'discord', 'groupPolicy'], form.discordGroupPolicy)
  setPathValue(mutable, ['channels', 'discord', 'replyToMode'], form.discordReplyToMode)

  const entries = parseDiscordGuildChannels(form.discordGuildChannels)
  const existingGuilds = asRecord(getPathValue(mutable, ['channels', 'discord', 'guilds'])) || {}

  if (entries.length === 0) {
    deletePathValue(mutable, ['channels', 'discord', 'guilds'])
    return
  }

  const guilds: JsonRecord = {}
  for (const entry of entries) {
    const existingGuild = asRecord(existingGuilds[entry.guildId])
    if (!guilds[entry.guildId]) {
      guilds[entry.guildId] = existingGuild ? { ...existingGuild } : {}
    }

    if (!entry.channelId) {
      continue
    }

    const guildValue = asRecord(guilds[entry.guildId]) || {}
    const existingChannels = asRecord(guildValue.channels)
    const nextChannels: JsonRecord = existingChannels ? { ...existingChannels } : {}

    const existingChannel = asRecord(nextChannels[entry.channelId])
    nextChannels[entry.channelId] = existingChannel ? { ...existingChannel } : { allow: true }
    guildValue.channels = nextChannels
    guilds[entry.guildId] = guildValue
  }

  setPathValue(mutable, ['channels', 'discord', 'guilds'], guilds)
}

const applySlackConfig = (mutable: JsonRecord, form: ChannelForm) => {
  setPathValue(mutable, ['channels', 'slack', 'enabled'], form.enabled)
  setPathValue(mutable, ['channels', 'slack', 'mode'], form.slackMode)
  setStringOrDelete(mutable, ['channels', 'slack', 'botToken'], form.slackBotToken)
  setStringOrDelete(mutable, ['channels', 'slack', 'appToken'], form.slackAppToken)
  setStringOrDelete(mutable, ['channels', 'slack', 'signingSecret'], form.slackSigningSecret)
  setNumberOrDelete(mutable, ['channels', 'slack', 'webhookPort'], form.slackWebhookPort, parsePositiveInt)
  setStringOrDelete(mutable, ['channels', 'slack', 'webhookPath'], form.slackWebhookPath)

  setPathValue(mutable, ['channels', 'slack', 'dmPolicy'], form.slackDmPolicy)
  setListOrDelete(mutable, ['channels', 'slack', 'allowFrom'], form.slackAllowFrom)
  setPathValue(mutable, ['channels', 'slack', 'groupPolicy'], form.slackGroupPolicy)

  const existingChannels = asRecord(getPathValue(mutable, ['channels', 'slack', 'channels'])) || {}
  const channelIds = parseListText(form.slackChannels)
  if (channelIds.length === 0) {
    deletePathValue(mutable, ['channels', 'slack', 'channels'])
  } else {
    const channels: JsonRecord = {}
    for (const channelId of channelIds) {
      const existing = asRecord(existingChannels[channelId])
      channels[channelId] = existing ? { ...existing } : { allow: true }
    }
    setPathValue(mutable, ['channels', 'slack', 'channels'], channels)
  }

  setPathValue(mutable, ['channels', 'slack', 'replyToMode'], form.slackReplyToMode)
  setStringOrDelete(mutable, ['channels', 'slack', 'defaultTo'], form.slackDefaultTo)
  setPathValue(mutable, ['channels', 'slack', 'requireMention'], form.slackRequireMention)
  setNumberOrDelete(
    mutable,
    ['channels', 'slack', 'textChunkLimit'],
    form.slackTextChunkLimit,
    parsePositiveInt
  )
  setStringOrDelete(mutable, ['channels', 'slack', 'chunkMode'], form.slackChunkMode)
}

const applyWhatsAppConfig = (mutable: JsonRecord, form: ChannelForm) => {
  setPathValue(mutable, ['channels', 'whatsapp', 'enabled'], form.enabled)
  setStringOrDelete(mutable, ['channels', 'whatsapp', 'sessionDir'], form.whatsappSessionDir)
  setPathValue(mutable, ['channels', 'whatsapp', 'useRemoteAuth'], form.whatsappUseRemoteAuth)
  setNumberOrDelete(
    mutable,
    ['channels', 'whatsapp', 'webhookPort'],
    form.whatsappWebhookPort,
    parsePositiveInt
  )
  setStringOrDelete(mutable, ['channels', 'whatsapp', 'webhookPath'], form.whatsappWebhookPath)

  setPathValue(mutable, ['channels', 'whatsapp', 'dmPolicy'], form.whatsappDmPolicy)
  setListOrDelete(mutable, ['channels', 'whatsapp', 'allowFrom'], form.whatsappAllowFrom)
  setPathValue(mutable, ['channels', 'whatsapp', 'groupPolicy'], form.whatsappGroupPolicy)
  setListOrDelete(mutable, ['channels', 'whatsapp', 'groupAllowFrom'], form.whatsappGroupAllowFrom)

  const existingGroups = asRecord(getPathValue(mutable, ['channels', 'whatsapp', 'groups'])) || {}
  const groupIds = parseListText(form.whatsappGroups)
  if (groupIds.length === 0) {
    deletePathValue(mutable, ['channels', 'whatsapp', 'groups'])
  } else {
    const groups: JsonRecord = {}
    for (const groupId of groupIds) {
      const existing = asRecord(existingGroups[groupId])
      groups[groupId] = existing ? { ...existing } : { enabled: true }
    }
    setPathValue(mutable, ['channels', 'whatsapp', 'groups'], groups)
  }

  setPathValue(
    mutable,
    ['channels', 'whatsapp', 'includeAttachments'],
    form.whatsappIncludeAttachments
  )
  setNumberOrDelete(
    mutable,
    ['channels', 'whatsapp', 'mediaMaxMb'],
    form.whatsappMediaMaxMb,
    parsePositiveNumber
  )
  setNumberOrDelete(
    mutable,
    ['channels', 'whatsapp', 'textChunkLimit'],
    form.whatsappTextChunkLimit,
    parsePositiveInt
  )
  setStringOrDelete(mutable, ['channels', 'whatsapp', 'chunkMode'], form.whatsappChunkMode)
  setPathValue(
    mutable,
    ['channels', 'whatsapp', 'sendReadReceipts'],
    form.whatsappSendReadReceipts
  )
}

const applyIMessageConfig = (mutable: JsonRecord, form: ChannelForm) => {
  setPathValue(mutable, ['channels', 'imessage', 'enabled'], form.enabled)
  setStringOrDelete(mutable, ['channels', 'imessage', 'cliPath'], form.imessageCliPath)
  setStringOrDelete(mutable, ['channels', 'imessage', 'dbPath'], form.imessageDbPath)
  setStringOrDelete(mutable, ['channels', 'imessage', 'remoteHost'], form.imessageRemoteHost)

  setPathValue(mutable, ['channels', 'imessage', 'dmPolicy'], form.imessageDmPolicy)
  setListOrDelete(mutable, ['channels', 'imessage', 'allowFrom'], form.imessageAllowFrom)
  setPathValue(mutable, ['channels', 'imessage', 'groupPolicy'], form.imessageGroupPolicy)
  setListOrDelete(mutable, ['channels', 'imessage', 'groupAllowFrom'], form.imessageGroupAllowFrom)

  const existingGroups = asRecord(getPathValue(mutable, ['channels', 'imessage', 'groups'])) || {}
  const groupIds = parseListText(form.imessageGroups)
  if (groupIds.length === 0) {
    deletePathValue(mutable, ['channels', 'imessage', 'groups'])
  } else {
    const groups: JsonRecord = {}
    for (const groupId of groupIds) {
      const existing = asRecord(existingGroups[groupId])
      groups[groupId] = existing ? { ...existing } : { enabled: true }
    }
    setPathValue(mutable, ['channels', 'imessage', 'groups'], groups)
  }

  setPathValue(
    mutable,
    ['channels', 'imessage', 'includeAttachments'],
    form.imessageIncludeAttachments
  )
  setListOrDelete(mutable, ['channels', 'imessage', 'attachmentRoots'], form.imessageAttachmentRoots)
  setListOrDelete(
    mutable,
    ['channels', 'imessage', 'remoteAttachmentRoots'],
    form.imessageRemoteAttachmentRoots
  )
  setNumberOrDelete(
    mutable,
    ['channels', 'imessage', 'mediaMaxMb'],
    form.imessageMediaMaxMb,
    parsePositiveNumber
  )
  setStringOrDelete(mutable, ['channels', 'imessage', 'service'], form.imessageService)
  setStringOrDelete(mutable, ['channels', 'imessage', 'region'], form.imessageRegion)
  setNumberOrDelete(
    mutable,
    ['channels', 'imessage', 'textChunkLimit'],
    form.imessageTextChunkLimit,
    parsePositiveInt
  )
  setStringOrDelete(mutable, ['channels', 'imessage', 'chunkMode'], form.imessageChunkMode)
}

const applyWecomConfig = (mutable: JsonRecord, form: ChannelForm) => {
  setPathValue(mutable, ['channels', 'wecom', 'enabled'], form.enabled)
  setStringOrDelete(mutable, ['channels', 'wecom', 'botId'], form.token)
  setStringOrDelete(mutable, ['channels', 'wecom', 'secret'], form.userId)
  setStringOrDelete(mutable, ['channels', 'wecom', 'name'], form.wecomName)
  setPathValue(mutable, ['channels', 'wecom', 'dmPolicy'], form.wecomDmPolicy)
  setListOrDelete(mutable, ['channels', 'wecom', 'allowFrom'], form.wecomAllowFrom)
  setPathValue(mutable, ['channels', 'wecom', 'groupPolicy'], form.wecomGroupPolicy)
  setListOrDelete(mutable, ['channels', 'wecom', 'groupAllowFrom'], form.wecomGroupAllowFrom)
  setStringOrDelete(mutable, ['channels', 'wecom', 'websocketUrl'], form.wecomWebsocketUrl)
  setPathValue(mutable, ['channels', 'wecom', 'sendThinkingMessage'], form.wecomSendThinkingMessage)

  const existingGroups = asRecord(getPathValue(mutable, ['channels', 'wecom', 'groups'])) || {}
  const groupIds = parseListText(form.wecomGroups)
  if (groupIds.length === 0) {
    deletePathValue(mutable, ['channels', 'wecom', 'groups'])
  } else {
    const groups: JsonRecord = {}
    for (const groupId of groupIds) {
      const existing = asRecord(existingGroups[groupId])
      groups[groupId] = existing ? { ...existing } : { enabled: true }
    }
    setPathValue(mutable, ['channels', 'wecom', 'groups'], groups)
  }
}

const applyQqConfig = (mutable: JsonRecord, form: ChannelForm) => {
  setPathValue(mutable, ['channels', 'qqbot', 'enabled'], form.enabled)
  setStringOrDelete(mutable, ['channels', 'qqbot', 'appId'], form.token)
  setStringOrDelete(mutable, ['channels', 'qqbot', 'clientSecret'], form.userId)
  deletePathValue(mutable, ['channels', 'qqbot', 'token'])
  deletePathValue(mutable, ['channels', 'qqbot', 'clientSecretFile'])
  setStringOrDelete(mutable, ['channels', 'qqbot', 'name'], form.qqName)
  setPathValue(mutable, ['channels', 'qqbot', 'dmPolicy'], form.qqDmPolicy)
  setListOrDelete(mutable, ['channels', 'qqbot', 'allowFrom'], form.qqAllowFrom)
  setStringOrDelete(mutable, ['channels', 'qqbot', 'systemPrompt'], form.qqSystemPrompt)
  setStringOrDelete(mutable, ['channels', 'qqbot', 'imageServerBaseUrl'], form.qqImageServerBaseUrl)
  setPathValue(mutable, ['channels', 'qqbot', 'markdownSupport'], form.qqMarkdownSupport)
}

const applyFeishuConfig = (mutable: JsonRecord, form: ChannelForm) => {
  ensureFeishuPluginAllowed(mutable)
  setPathValue(mutable, ['channels', 'feishu', 'enabled'], form.enabled)
  setStringOrDelete(mutable, ['channels', 'feishu', 'appId'], form.token)
  setStringOrDelete(mutable, ['channels', 'feishu', 'appSecret'], form.userId)
  setStringOrDelete(mutable, ['channels', 'feishu', 'domain'], form.feishuDomain)
  setPathValue(mutable, ['channels', 'feishu', 'connectionMode'], form.feishuConnectionMode)

  setPathValue(mutable, ['channels', 'feishu', 'dmPolicy'], form.feishuDmPolicy)
  setListOrDelete(mutable, ['channels', 'feishu', 'allowFrom'], form.feishuAllowFrom)
  setPathValue(mutable, ['channels', 'feishu', 'groupPolicy'], form.feishuGroupPolicy)
  setListOrDelete(mutable, ['channels', 'feishu', 'groupAllowFrom'], form.feishuGroupAllowFrom)
  setPathValue(
    mutable,
    ['channels', 'feishu', 'groupCommandMentionBypass'],
    form.feishuGroupCommandMentionBypass
  )

  setStringOrDelete(mutable, ['channels', 'feishu', 'webhookPath'], form.feishuWebhookPath)
  setNumberOrDelete(mutable, ['channels', 'feishu', 'webhookPort'], form.feishuWebhookPort, parsePositiveInt)
  setStringOrDelete(mutable, ['channels', 'feishu', 'encryptKey'], form.feishuEncryptKey)
  setStringOrDelete(
    mutable,
    ['channels', 'feishu', 'verificationToken'],
    form.feishuVerificationToken
  )

  setPathValue(mutable, ['channels', 'feishu', 'renderMode'], form.feishuRenderMode)
  setNumberOrDelete(mutable, ['channels', 'feishu', 'mediaMaxMb'], form.feishuMediaMaxMb, parsePositiveNumber)

  const dynamicAgentCreation: JsonRecord = {
    enabled: form.feishuDynamicEnabled
  }
  const workspaceTemplate = form.feishuDynamicWorkspaceTemplate.trim()
  const agentDirTemplate = form.feishuDynamicAgentDirTemplate.trim()
  const maxAgents = parsePositiveInt(form.feishuDynamicMaxAgents)
  if (workspaceTemplate) dynamicAgentCreation.workspaceTemplate = workspaceTemplate
  if (agentDirTemplate) dynamicAgentCreation.agentDirTemplate = agentDirTemplate
  if (typeof maxAgents === 'number') dynamicAgentCreation.maxAgents = maxAgents

  setPathValue(mutable, ['channels', 'feishu', 'dynamicAgentCreation'], dynamicAgentCreation)
}

const applyDingtalkConfig = (mutable: JsonRecord, form: ChannelForm) => {
  ensureDingtalkPluginAllowed(mutable)
  const channels = ensureRecord(mutable, 'channels')
  const dingtalkConfig = ensureDingtalkChannelConfigNode(channels)
  const nextConfig: JsonRecord = {
    enabled: form.enabled,
    dmPolicy: form.dingtalkDmPolicy,
    groupPolicy: form.dingtalkGroupPolicy,
    debug: form.dingtalkDebug,
    showThinking: form.dingtalkShowThinking,
    messageType: form.dingtalkMessageType,
    cardTemplateKey: form.dingtalkCardTemplateKey.trim() || 'content',
  }

  const name = form.dingtalkName.trim()
  const clientId = form.token.trim()
  const clientSecret = form.userId.trim()
  const robotCode = form.dingtalkRobotCode.trim()
  const corpId = form.dingtalkCorpId.trim()
  const agentId = form.dingtalkAgentId.trim()
  const cardTemplateId = form.dingtalkCardTemplateId.trim()
  const allowFrom = parseListText(form.dingtalkAllowFrom)
  const mediaUrlAllowlist = parseListText(form.dingtalkMediaUrlAllowlist)
  const mediaMaxMb = parsePositiveNumber(form.dingtalkMediaMaxMb)

  if (name) nextConfig.name = name
  if (clientId) nextConfig.clientId = clientId
  if (clientSecret) nextConfig.clientSecret = clientSecret
  if (robotCode) nextConfig.robotCode = robotCode
  if (corpId) nextConfig.corpId = corpId
  if (agentId) nextConfig.agentId = agentId
  if (cardTemplateId) nextConfig.cardTemplateId = cardTemplateId
  if (allowFrom.length > 0) nextConfig.allowFrom = allowFrom
  if (mediaUrlAllowlist.length > 0) nextConfig.mediaUrlAllowlist = mediaUrlAllowlist
  if (typeof mediaMaxMb === 'number') nextConfig.mediaMaxMb = mediaMaxMb

  channels[DINGTALK_CHANNEL_KEY] = {
    ...mergeDingtalkEditableConfig(dingtalkConfig, nextConfig),
  }
}

const syncChannelsFromConfig = async () => {
  try {
    const [config] = await loadLocalConfig()
    const root = config as JsonRecord
    const channelsRaw = asRecord(root.channels) || {}
    const bindings = parseBindings(root)
    const agentOptions = parseAgentOptions(root)
    availableAgents.value = agentOptions

    for (const channelId of channelIds) {
      const channelNode = getChannelConfigNode(channelsRaw, channelId)
      const accountIds = collectChannelAccountIds(channelNode, bindings, channelId, agentOptions)
      availableAccountsByChannel.value[channelId] = accountIds.map(accountId => {
        const accountNode = accountId === 'default' ? channelNode : getAccountNode(channelNode, accountId)
        const label =
          channelId === 'telegram' ||
          channelId === 'discord' ||
          channelId === 'slack' ||
          channelId === 'feishu' ||
          channelId === 'wecom' ||
          channelId === 'qq' ||
          channelId === 'dingtalk'
            ? buildMessageChannelAccountLabel(
                channelId as ManagedMessageChannelId,
                accountId,
                accountNode
              )
            : accountId

        return {
          id: accountId,
          label,
        }
      })
      if (
        channelId === 'telegram' ||
        channelId === 'discord' ||
        channelId === 'slack' ||
        channelId === 'feishu' ||
        channelId === 'wecom' ||
        channelId === 'qq' ||
        channelId === 'dingtalk'
      ) {
        configuredByChannel.value[channelId] = isMessageChannelConfigured(
          channelId as ManagedMessageChannelId,
          channelNode
        )
      } else {
        configuredByChannel.value[channelId] = false
      }

      const previousAgent = selectedAgentByChannel.value[channelId]
      const nextAgent = agentOptions.some(agent => agent.id === previousAgent)
        ? previousAgent
        : agentOptions[0]?.id || 'default'
      selectedAgentByChannel.value[channelId] = nextAgent

      const boundAccountId = resolveBindingAccountId(bindings, channelId, nextAgent)
      const previousAccountId = selectedAccountByChannel.value[channelId]
      const channelDefaultAccountId = resolveChannelDefaultAccountId(channelNode, accountIds)
      const nextAccountId =
        (boundAccountId && accountIds.includes(boundAccountId)
          ? boundAccountId
          : accountIds.includes(previousAccountId)
            ? previousAccountId
            : channelDefaultAccountId) || 'default'
      selectedAccountByChannel.value[channelId] = nextAccountId

      const enabledValue = readChannelValue(channelNode, nextAccountId, ['enabled'])
      forms.value[channelId].enabled =
        typeof enabledValue === 'boolean' ? enabledValue : Boolean(channelNode.enabled)
    }

    const telegram = getChannelConfigNode(channelsRaw, 'telegram')
    const readTelegramAccount = (path: string[]) =>
      readChannelValue(telegram, getSelectedAccountId('telegram'), path)
    const readTelegramShared = (path: string[]) => readSharedChannelValue(telegram, path)
    forms.value.telegram.token = asString(readTelegramAccount(['botToken']))
    forms.value.telegram.telegramDmPolicy = enumOrDefault(
      readTelegramShared(['dmPolicy']),
      ['pairing', 'allowlist', 'open', 'disabled'],
      'pairing'
    )
    forms.value.telegram.telegramAllowFrom = listToText(readTelegramShared(['allowFrom']))
    forms.value.telegram.telegramGroupPolicy = enumOrDefault(
      readTelegramShared(['groupPolicy']),
      ['allowlist', 'open', 'disabled'],
      'allowlist'
    )
    forms.value.telegram.telegramGroupAllowFrom = listToText(readTelegramShared(['groupAllowFrom']))
    forms.value.telegram.telegramReplyToMode = enumOrDefault(
      readTelegramShared(['replyToMode']),
      ['off', 'first', 'all'],
      'off'
    )
    forms.value.telegram.telegramDefaultTo = asString(readTelegramShared(['defaultTo']))
    forms.value.telegram.telegramGroups = buildObjectKeysText(readTelegramShared(['groups']))

    const discord = getChannelConfigNode(channelsRaw, 'discord')
    const readDiscordAccount = (path: string[]) =>
      readChannelValue(discord, getSelectedAccountId('discord'), path)
    const readDiscordShared = (path: string[]) => readSharedChannelValue(discord, path)
    forms.value.discord.token = asString(readDiscordAccount(['token']))
    forms.value.discord.discordDmPolicy = enumOrDefault(
      readDiscordShared(['dm', 'policy']) ?? readDiscordShared(['dmPolicy']),
      ['pairing', 'allowlist', 'open', 'disabled'],
      'pairing'
    )
    forms.value.discord.discordAllowFrom = listToText(
      readDiscordShared(['dm', 'allowFrom']) ?? readDiscordShared(['allowFrom'])
    )
    forms.value.discord.discordGroupPolicy = enumOrDefault(
      readDiscordShared(['groupPolicy']),
      ['allowlist', 'open', 'disabled'],
      'allowlist'
    )
    forms.value.discord.discordGuildChannels = buildDiscordGuildChannelsText(readDiscordShared(['guilds']))
    forms.value.discord.discordReplyToMode = enumOrDefault(
      readDiscordShared(['replyToMode']),
      ['off', 'first', 'all'],
      'off'
    )

    const slack = getChannelConfigNode(channelsRaw, 'slack')
    const readSlackAccount = (path: string[]) => readChannelValue(slack, getSelectedAccountId('slack'), path)
    const readSlackShared = (path: string[]) => readSharedChannelValue(slack, path)
    forms.value.slack.slackMode = enumOrDefault(readSlackShared(['mode']), ['http', 'socket'], 'http')
    forms.value.slack.slackBotToken = asString(readSlackAccount(['botToken']))
    forms.value.slack.slackAppToken = asString(readSlackAccount(['appToken']))
    forms.value.slack.slackSigningSecret = asString(readSlackAccount(['signingSecret']))
    forms.value.slack.slackWebhookPort = asString(readSlackShared(['webhookPort']))
    forms.value.slack.slackWebhookPath = asString(readSlackShared(['webhookPath'])) || '/webhooks/slack'
    forms.value.slack.slackDmPolicy = enumOrDefault(
      readSlackShared(['dmPolicy']),
      ['pairing', 'allowlist', 'open', 'disabled'],
      'pairing'
    )
    forms.value.slack.slackAllowFrom = listToText(readSlackShared(['allowFrom']))
    forms.value.slack.slackGroupPolicy = enumOrDefault(
      readSlackShared(['groupPolicy']),
      ['allowlist', 'open', 'disabled'],
      'allowlist'
    )
    forms.value.slack.slackChannels = buildObjectKeysText(readSlackShared(['channels']))
    forms.value.slack.slackReplyToMode = enumOrDefault(
      readSlackShared(['replyToMode']),
      ['off', 'first', 'all'],
      'off'
    )
    forms.value.slack.slackDefaultTo = asString(readSlackShared(['defaultTo']))
    forms.value.slack.slackRequireMention =
      typeof readSlackShared(['requireMention']) === 'boolean' ? (readSlackShared(['requireMention']) as boolean) : true
    forms.value.slack.slackTextChunkLimit = asString(readSlackShared(['textChunkLimit']))
    forms.value.slack.slackChunkMode = asString(readSlackShared(['chunkMode'])) || 'sentence'

    const whatsapp = getChannelConfigNode(channelsRaw, 'whatsapp')
    const readWhatsApp = (path: string[]) =>
      readChannelValue(whatsapp, getSelectedAccountId('whatsapp'), path)
    forms.value.whatsapp.whatsappSessionDir = asString(readWhatsApp(['sessionDir']))
    forms.value.whatsapp.whatsappUseRemoteAuth =
      typeof readWhatsApp(['useRemoteAuth']) === 'boolean'
        ? (readWhatsApp(['useRemoteAuth']) as boolean)
        : false
    forms.value.whatsapp.whatsappWebhookPort = asString(readWhatsApp(['webhookPort']))
    forms.value.whatsapp.whatsappWebhookPath =
      asString(readWhatsApp(['webhookPath'])) || '/webhooks/whatsapp'
    forms.value.whatsapp.whatsappDmPolicy = enumOrDefault(
      readWhatsApp(['dmPolicy']),
      ['pairing', 'allowlist', 'open', 'disabled'],
      'pairing'
    )
    forms.value.whatsapp.whatsappAllowFrom = listToText(readWhatsApp(['allowFrom']))
    forms.value.whatsapp.whatsappGroupPolicy = enumOrDefault(
      readWhatsApp(['groupPolicy']),
      ['allowlist', 'open', 'disabled'],
      'allowlist'
    )
    forms.value.whatsapp.whatsappGroupAllowFrom = listToText(readWhatsApp(['groupAllowFrom']))
    forms.value.whatsapp.whatsappGroups = buildObjectKeysText(readWhatsApp(['groups']))
    forms.value.whatsapp.whatsappIncludeAttachments =
      typeof readWhatsApp(['includeAttachments']) === 'boolean'
        ? (readWhatsApp(['includeAttachments']) as boolean)
        : true
    forms.value.whatsapp.whatsappMediaMaxMb = asString(readWhatsApp(['mediaMaxMb']))
    forms.value.whatsapp.whatsappTextChunkLimit = asString(readWhatsApp(['textChunkLimit']))
    forms.value.whatsapp.whatsappChunkMode = asString(readWhatsApp(['chunkMode'])) || 'sentence'
    forms.value.whatsapp.whatsappSendReadReceipts = Boolean(readWhatsApp(['sendReadReceipts']))

    const imessage = getChannelConfigNode(channelsRaw, 'imessage')
    const readIMessage = (path: string[]) =>
      readChannelValue(imessage, getSelectedAccountId('imessage'), path)
    forms.value.imessage.imessageCliPath = asString(readIMessage(['cliPath'])) || 'rustpush'
    forms.value.imessage.imessageDbPath = asString(readIMessage(['dbPath']))
    forms.value.imessage.imessageRemoteHost = asString(readIMessage(['remoteHost']))
    forms.value.imessage.imessageDmPolicy = enumOrDefault(
      readIMessage(['dmPolicy']),
      ['pairing', 'allowlist', 'open', 'disabled'],
      'pairing'
    )
    forms.value.imessage.imessageAllowFrom = listToText(readIMessage(['allowFrom']))
    forms.value.imessage.imessageGroupPolicy = enumOrDefault(
      readIMessage(['groupPolicy']),
      ['allowlist', 'open', 'disabled'],
      'allowlist'
    )
    forms.value.imessage.imessageGroupAllowFrom = listToText(readIMessage(['groupAllowFrom']))
    forms.value.imessage.imessageGroups = buildObjectKeysText(readIMessage(['groups']))
    forms.value.imessage.imessageIncludeAttachments =
      typeof readIMessage(['includeAttachments']) === 'boolean'
        ? (readIMessage(['includeAttachments']) as boolean)
        : true
    forms.value.imessage.imessageAttachmentRoots = listToText(readIMessage(['attachmentRoots']))
    forms.value.imessage.imessageRemoteAttachmentRoots = listToText(
      readIMessage(['remoteAttachmentRoots'])
    )
    forms.value.imessage.imessageMediaMaxMb = asString(readIMessage(['mediaMaxMb']))
    forms.value.imessage.imessageService = asString(readIMessage(['service'])) || 'auto'
    forms.value.imessage.imessageRegion = asString(readIMessage(['region'])) || 'CN'
    forms.value.imessage.imessageTextChunkLimit = asString(readIMessage(['textChunkLimit']))
    forms.value.imessage.imessageChunkMode = asString(readIMessage(['chunkMode'])) || 'sentence'

    const wecom = getChannelConfigNode(channelsRaw, 'wecom')
    const readWecomAccount = (path: string[]) => readChannelValue(wecom, getSelectedAccountId('wecom'), path)
    const readWecomShared = (path: string[]) => readSharedChannelValue(wecom, path)
    forms.value.wecom.token = asString(readWecomAccount(['botId']))
    forms.value.wecom.userId = asString(readWecomAccount(['secret']))
    forms.value.wecom.wecomName = asString(readWecomAccount(['name']))
    forms.value.wecom.wecomDmPolicy = enumOrDefault(
      readWecomShared(['dmPolicy']),
      ['pairing', 'allowlist', 'open', 'disabled'],
      'pairing'
    )
    forms.value.wecom.wecomAllowFrom = listToText(readWecomShared(['allowFrom']))
    forms.value.wecom.wecomGroupPolicy = enumOrDefault(
      readWecomShared(['groupPolicy']),
      ['allowlist', 'open', 'disabled'],
      'open'
    )
    forms.value.wecom.wecomGroupAllowFrom = listToText(readWecomShared(['groupAllowFrom']))
    forms.value.wecom.wecomGroups = buildObjectKeysText(readWecomShared(['groups']))
    forms.value.wecom.wecomWebsocketUrl = asString(readWecomShared(['websocketUrl']))
    forms.value.wecom.wecomSendThinkingMessage =
      typeof readWecomShared(['sendThinkingMessage']) === 'boolean'
        ? (readWecomShared(['sendThinkingMessage']) as boolean)
        : true

    const qq = getChannelConfigNode(channelsRaw, 'qq')
    const readQqAccount = (path: string[]) => readChannelValue(qq, getSelectedAccountId('qq'), path)
    const readQqShared = (path: string[]) => readSharedChannelValue(qq, path)
    const qqCredentials = parseQqCredentialsFromConfig(
      asRecord(readChannelValue(qq, getSelectedAccountId('qq'), [])) || asRecord(qq)
    )
    forms.value.qq.token = qqCredentials.appId
    forms.value.qq.userId = qqCredentials.clientSecret
    forms.value.qq.qqName = asString(readQqAccount(['name']))
    forms.value.qq.qqDmPolicy = enumOrDefault(
      readQqShared(['dmPolicy']),
      ['open', 'pairing', 'allowlist'],
      'pairing'
    )
    forms.value.qq.qqAllowFrom = listToText(readQqShared(['allowFrom']))
    forms.value.qq.qqSystemPrompt = asString(readQqShared(['systemPrompt']))
    forms.value.qq.qqImageServerBaseUrl = asString(readQqShared(['imageServerBaseUrl']))
    forms.value.qq.qqMarkdownSupport =
      typeof readQqShared(['markdownSupport']) === 'boolean'
        ? (readQqShared(['markdownSupport']) as boolean)
        : true

    const feishu = getChannelConfigNode(channelsRaw, 'feishu')
    const readFeishuAccount = (path: string[]) => readChannelValue(feishu, getSelectedAccountId('feishu'), path)
    const readFeishuShared = (path: string[]) => readSharedChannelValue(feishu, path)
    const feishuDynamic = asRecord(readFeishuShared(['dynamicAgentCreation'])) || {}
    forms.value.feishu.token = asString(readFeishuAccount(['appId']))
    forms.value.feishu.userId = asString(readFeishuAccount(['appSecret']))
    forms.value.feishu.feishuDomain = asString(readFeishuShared(['domain'])) || 'feishu'
    forms.value.feishu.feishuConnectionMode = enumOrDefault(
      readFeishuShared(['connectionMode']),
      ['websocket', 'webhook'],
      'websocket'
    )
    forms.value.feishu.feishuDmPolicy = enumOrDefault(
      readFeishuShared(['dmPolicy']),
      ['pairing', 'allowlist', 'open'],
      'pairing'
    )
    forms.value.feishu.feishuAllowFrom = listToText(readFeishuShared(['allowFrom']))
    forms.value.feishu.feishuGroupPolicy = enumOrDefault(
      readFeishuShared(['groupPolicy']),
      ['allowlist', 'open', 'disabled'],
      'allowlist'
    )
    forms.value.feishu.feishuGroupAllowFrom = listToText(readFeishuShared(['groupAllowFrom']))
    forms.value.feishu.feishuGroupCommandMentionBypass = enumOrDefault(
      readFeishuShared(['groupCommandMentionBypass']),
      ['single_bot', 'never', 'always'],
      'single_bot'
    )
    forms.value.feishu.feishuWebhookPath = asString(readFeishuShared(['webhookPath'])) || '/feishu/events'
    forms.value.feishu.feishuWebhookPort = asString(readFeishuShared(['webhookPort']))
    forms.value.feishu.feishuEncryptKey = asString(readFeishuShared(['encryptKey']))
    forms.value.feishu.feishuVerificationToken = asString(readFeishuShared(['verificationToken']))
    forms.value.feishu.feishuRenderMode = enumOrDefault(
      readFeishuShared(['renderMode']),
      ['auto', 'raw', 'card'],
      'auto'
    )
    forms.value.feishu.feishuMediaMaxMb = asString(readFeishuShared(['mediaMaxMb']))
    forms.value.feishu.feishuDynamicEnabled = Boolean(feishuDynamic.enabled)
    forms.value.feishu.feishuDynamicWorkspaceTemplate = asString(feishuDynamic.workspaceTemplate)
    forms.value.feishu.feishuDynamicAgentDirTemplate = asString(feishuDynamic.agentDirTemplate)
    forms.value.feishu.feishuDynamicMaxAgents = asString(feishuDynamic.maxAgents)

    const dingtalk = getChannelConfigNode(channelsRaw, 'dingtalk')
    const readDingtalkAccount = (path: string[]) =>
      readChannelValue(dingtalk, getSelectedAccountId('dingtalk'), path)
    const readDingtalkShared = (path: string[]) => readSharedChannelValue(dingtalk, path)
    forms.value.dingtalk.token = asString(readDingtalkAccount(['clientId']))
    forms.value.dingtalk.userId = asString(readDingtalkAccount(['clientSecret']))
    forms.value.dingtalk.dingtalkDmPolicy = enumOrDefault(
      readDingtalkShared(['dmPolicy']),
      ['open', 'pairing', 'allowlist'],
      'open'
    )
    forms.value.dingtalk.dingtalkAllowFrom = listToText(readDingtalkShared(['allowFrom']))
    forms.value.dingtalk.dingtalkGroupPolicy = enumOrDefault(
      readDingtalkShared(['groupPolicy']),
      ['open', 'allowlist'],
      'open'
    )
    forms.value.dingtalk.dingtalkName = asString(readDingtalkAccount(['name']))
    forms.value.dingtalk.dingtalkRobotCode = asString(readDingtalkAccount(['robotCode']))
    forms.value.dingtalk.dingtalkCorpId = asString(readDingtalkAccount(['corpId']))
    forms.value.dingtalk.dingtalkAgentId = asString(readDingtalkAccount(['agentId']))
    forms.value.dingtalk.dingtalkMessageType = enumOrDefault(
      readDingtalkShared(['messageType']),
      ['markdown', 'card'],
      'markdown'
    )
    forms.value.dingtalk.dingtalkCardTemplateId = asString(readDingtalkShared(['cardTemplateId']))
    forms.value.dingtalk.dingtalkCardTemplateKey =
      asString(readDingtalkShared(['cardTemplateKey'])) || 'content'
    forms.value.dingtalk.dingtalkMediaUrlAllowlist = listToText(readDingtalkShared(['mediaUrlAllowlist']))
    forms.value.dingtalk.dingtalkShowThinking =
      typeof readDingtalkShared(['showThinking']) === 'boolean'
        ? (readDingtalkShared(['showThinking']) as boolean)
        : true
    forms.value.dingtalk.dingtalkMediaMaxMb = asString(readDingtalkShared(['mediaMaxMb']))
    forms.value.dingtalk.dingtalkDebug = Boolean(readDingtalkShared(['debug']))
  } catch {
    // ignore, keep local values
  }
}

const persistConfigMutation = async (mutator: (mutable: JsonRecord) => void) => {
  const [config, info] = await loadLocalConfig()
  const mutable = config as JsonRecord
  mutator(mutable)
  await invoke('save_config', {
    config: mutable,
    path: info.path
  })
}

const persistChannelEnabled = async (channelId: ChannelId, enabled: boolean) => {
  await persistConfigMutation(mutable => {
    setPathValue(mutable, ['channels', channelConfigKey(channelId), 'enabled'], enabled)
  })
}

const openPairingModal = () => {
  if (selectedChannelId.value !== 'feishu' || !selectedExtensionInstalled.value) return
  pairingInput.value = ''
  showPairingCodeModal.value = true
}

const closePairingModal = () => {
  if (approvingPairing.value) return
  showPairingCodeModal.value = false
}

const submitPairing = async () => {
  const pairingCode = extractFeishuPairingCode(pairingInput.value)
  if (!pairingCode) {
    props.showToast('error', '请先填写有效配对码')
    return
  }

  approvingPairing.value = true
  try {
    await invoke<string>('approve_feishu_pairing', { pairingCode })
    showPairingCodeModal.value = false
    pairingInput.value = ''
    props.showToast('success', '配对成功')
  } catch (error) {
    props.showToast('error', String(error))
  } finally {
    approvingPairing.value = false
  }
}

const handlePairingInputChange = (value: string) => {
  pairingInput.value = value
}

const openAppCenter = async () => {
  let url = ''
  if (selectedChannelId.value === 'feishu') {
    url = 'https://open.feishu.cn/app'
  } else if (selectedChannelId.value === 'dingtalk') {
    url = 'https://open-dev.dingtalk.com/fe/app?hash=%23%2Fcorp%2Fapp#/corp/app'
  } else {
    return
  }

  try {
    await openExternal(url)
  } catch (error) {
    props.showToast('error', `打开 ${selectedChannel.value.name}开放平台失败: ${String(error)}`)
  }
}

const scheduleGatewayRestartAfterChannelEnable = (channelName: string) => {
  const restartCommand = resolveGatewayRestartCommand(props.systemOs)

  gatewayRestartController.schedule(async () => {
    try {
      await invoke<string>('restart_gateway')
      props.showToast('success', `已为 ${channelName} 发送网关重启命令：${restartCommand}`)
    } catch (error) {
      props.showToast('error', `网关重启失败：${String(error)}`)
    }
  })
}

const toggleChannelEnabled = async () => {
  if (!canConfigureCurrentChannel.value) {
    props.showToast('error', `${selectedChannel.value.name} 扩展未安装，暂不可配置`)
    return
  }

  const channelId = selectedChannelId.value
  const selectedAgentId = getSelectedAgentId(channelId)
  const selectedAccountId = getSelectedAccountId(channelId)
  const accountMode = isAccountModeEnabled(channelId)
  const previous = currentForm.value.enabled
  const next = !previous
  currentForm.value.enabled = next

  try {
    if (accountMode) {
      if (
        next &&
        (channelId === 'feishu' || channelId === 'wecom' || channelId === 'dingtalk') &&
        (!currentForm.value.token.trim() || !currentForm.value.userId.trim())
      ) {
        throw new Error(`${selectedChannel.value.name} 启用前需要先填写应用凭据`)
      }

      if (next && channelId === 'qq' && (!currentForm.value.token.trim() || !currentForm.value.userId.trim())) {
        throw new Error(`${selectedChannel.value.name} 启用前需要先填写 App ID 和 App Secret`)
      }

      await persistConfigMutation(mutable => {
        const key = channelConfigKey(channelId)
        if (channelId === 'dingtalk') {
          const channels = ensureRecord(mutable, 'channels')
          ensureDingtalkPluginAllowed(mutable)
          ensureDingtalkChannelConfigNode(channels)
        }
        if (selectedAccountId === 'default') {
          setPathValue(mutable, ['channels', key, 'enabled'], next)
        } else {
          setPathValue(mutable, ['channels', key, 'accounts', selectedAccountId, 'enabled'], next)
        }
        syncChannelEnabledFromAccounts(mutable, channelId, next)
        upsertAgentBinding(mutable, channelId, selectedAgentId, selectedAccountId)
      })
    } else if (channelId === 'feishu') {
      if (!currentForm.value.token.trim() || !currentForm.value.userId.trim()) {
        throw new Error('请先填写飞书 App ID 和 App Secret')
      }
      await persistConfigMutation(mutable => {
        mergeFeishuChannelConfig(mutable, {
          appId: currentForm.value.token.trim(),
          appSecret: currentForm.value.userId.trim(),
          enabled: next,
        })
      })
    } else if (channelId === 'dingtalk') {
      if (!currentForm.value.token.trim() || !currentForm.value.userId.trim()) {
        throw new Error('请先填写钉钉 Client ID 和 Client Secret')
      }
      await invoke<string>('set_dingtalk_channel_config', {
        clientId: currentForm.value.token.trim(),
        clientSecret: currentForm.value.userId.trim(),
        enabled: next
      })
    } else {
      await persistChannelEnabled(channelId, next)
    }

    await syncChannelsFromConfig()
    props.showToast('success', `${selectedChannel.value.name}已${next ? '启用' : '停用'}`)
    if (next) {
      scheduleGatewayRestartAfterChannelEnable(selectedChannel.value.name)
    }
  } catch (error) {
    currentForm.value.enabled = previous
    props.showToast('error', String(error))
  }
}

const refreshExtensionStatus = async () => {
  extensionStatusLoading.value = true
  try {
    extensionStatus.value = await invoke<ChannelExtensionStatus>('get_channel_extension_status')
  } catch (error) {
    props.showToast('error', `读取扩展状态失败: ${String(error)}`)
  } finally {
    extensionStatusLoading.value = false
  }
}

const openInstallModal = async () => {
  if (!isExtensionChannel(selectedChannelId.value)) return

  installingChannel.value = selectedChannelId.value
  installLogs.value = []
  showInstallModal.value = true
  installingExtension.value = true

  try {
    const installArgs: {
      channelId: ExtensionChannelId
    } = {
      channelId: installingChannel.value,
    }

    await invoke<string>('install_channel_extension', installArgs)
    props.showToast('success', `${selectedChannel.value.name} 扩展安装完成`)
    await refreshExtensionStatus()
  } catch (error) {
    props.showToast('error', String(error))
  } finally {
    installingExtension.value = false
  }
}

const closeInstallModal = () => {
  if (installingExtension.value) return
  showInstallModal.value = false
}

const saveConfig = async () => {
  if (!canConfigureCurrentChannel.value) {
    props.showToast('error', `${selectedChannel.value.name} 扩展未安装，暂不可配置`)
    return
  }

  const channelId = selectedChannelId.value
  const current = currentForm.value
  const selectedAgentId = getSelectedAgentId(channelId)
  const selectedAccountId = getSelectedAccountId(channelId)
  const accountMode = isAccountModeEnabled(channelId)

  try {
    if (isManagedChannel(channelId)) {
      const managedChannelId = channelId
      const managedApplyChannelConfig =
        managedChannelId === 'telegram'
          ? applyTelegramConfig
          : managedChannelId === 'discord'
            ? applyDiscordConfig
            : managedChannelId === 'slack'
              ? applySlackConfig
              : managedChannelId === 'wecom'
                ? applyWecomConfig
                : managedChannelId === 'qq'
                  ? applyQqConfig
              : managedChannelId === 'feishu'
                ? applyFeishuConfig
                : applyDingtalkConfig

      if (selectedPanel.value === 'credentials') {
        if ((managedChannelId === 'telegram' || managedChannelId === 'discord') && current.enabled && !current.token.trim()) {
          props.showToast('error', `${selectedChannel.value.name} 已启用时需要填写 Bot Token`)
          return
        }

        if (managedChannelId === 'slack' && current.enabled && !current.slackBotToken.trim()) {
          props.showToast('error', 'Slack 已启用时需要填写 Bot Token')
          return
        }

        if (
          (managedChannelId === 'feishu' ||
            managedChannelId === 'wecom' ||
            managedChannelId === 'dingtalk') &&
          (!current.token.trim() || !current.userId.trim())
        ) {
          props.showToast('error', `${selectedChannel.value.name} 需要先填写应用凭据`)
          return
        }

        if (managedChannelId === 'qq' && (!current.token.trim() || !current.userId.trim())) {
          props.showToast('error', 'QQ 需要先填写 App ID 和 App Secret')
          return
        }

        const draftChannel = buildManagedChannelDraftNode(
          managedChannelId,
          current,
          managedApplyChannelConfig
        )

        if (managedChannelId === 'dingtalk' && selectedAccountId === 'default') {
          await invoke<string>('set_dingtalk_channel_config', {
            clientId: current.token.trim(),
            clientSecret: current.userId.trim(),
            enabled: current.enabled
          })
        }

        await persistConfigMutation(mutable => {
          if (managedChannelId === 'dingtalk') {
            ensureDingtalkPluginAllowed(mutable)
          }

          if (selectedAccountId === 'default') {
            saveMessageChannelDefaultAccountConfig(mutable, managedChannelId, draftChannel)
          } else {
            saveMessageChannelAccountConfig(mutable, managedChannelId, selectedAccountId, draftChannel)
            upsertAgentBinding(mutable, managedChannelId, selectedAgentId, selectedAccountId)
          }

          syncChannelEnabledFromAccounts(mutable, managedChannelId, current.enabled)
        })

        await syncChannelsFromConfig()
        props.showToast('success', `${selectedChannel.value.name}账号配置已保存`)
        return
      }

      const draftChannel = buildManagedChannelDraftNode(managedChannelId, current, managedApplyChannelConfig)
      await persistConfigMutation(mutable => {
        if (managedChannelId === 'dingtalk') {
          ensureDingtalkPluginAllowed(mutable)
        }
        saveMessageChannelPublicConfig(mutable, managedChannelId, draftChannel)
      })
      await syncChannelsFromConfig()
      props.showToast('success', `${selectedChannel.value.name}公共配置已保存`)
      return
    }

    if (channelId === 'whatsapp') {
      if (accountMode) {
        await persistConfigMutation(mutable => {
          applyChannelConfigToAccount(mutable, channelId, selectedAccountId, current, applyWhatsAppConfig)
          upsertAgentBinding(mutable, channelId, selectedAgentId, selectedAccountId)
        })
        await syncChannelsFromConfig()
        props.showToast('success', 'WhatsApp 配置已保存')
        return
      }

      await persistConfigMutation(mutable => {
        applyWhatsAppConfig(mutable, current)
      })
      await syncChannelsFromConfig()
      props.showToast('success', 'WhatsApp 配置已保存')
      return
    }

    if (channelId === 'imessage') {
      if (accountMode) {
        await persistConfigMutation(mutable => {
          applyChannelConfigToAccount(mutable, channelId, selectedAccountId, current, applyIMessageConfig)
          upsertAgentBinding(mutable, channelId, selectedAgentId, selectedAccountId)
        })
        await syncChannelsFromConfig()
        props.showToast('success', 'iMessage 配置已保存')
        return
      }

      await persistConfigMutation(mutable => {
        applyIMessageConfig(mutable, current)
      })
      await syncChannelsFromConfig()
      props.showToast('success', 'iMessage 配置已保存')
      return
    }

    await persistChannelEnabled(channelId, current.enabled)
    await syncChannelsFromConfig()
    props.showToast('success', `${selectedChannel.value.name} 配置已保存`)
  } catch (error) {
    props.showToast('error', String(error))
  }
}

const handleClickOutside = (event: MouseEvent) => {
  const target = event.target as HTMLElement | null
  if (!target?.closest('.message-channel-account-dropdown')) {
    showAccountSelectorDropdown.value = false
  }
}

onMounted(async () => {
  document.addEventListener('click', handleClickOutside)
  await syncChannelsFromConfig()
  await refreshExtensionStatus()

  unlistenExtensionInstallLog = await listen<InstallLogEvent>('channel-extension-install-log', event => {
    if (!installingChannel.value || event.payload.step !== installingChannel.value) return
    installLogs.value.push(event.payload)
  })

  unlistenExtensionInstallState = await listen<ChannelExtensionInstallStateEvent>(
    'channel-extension-install-state',
    event => {
      if (!installingChannel.value || event.payload.channelId !== installingChannel.value) return
      if (event.payload.status === 'success' || event.payload.status === 'error') {
        installingExtension.value = false
      }
    }
  )
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
  unlistenExtensionInstallLog?.()
  unlistenExtensionInstallState?.()
  gatewayRestartController.dispose()
})
</script>

<template>
  <div class="oc-page-root h-full min-h-0">
    <div class="grid h-full min-h-0 grid-cols-1 gap-3 lg:grid-cols-[292px_minmax(0,1fr)]">
      <section class="oc-panel flex min-h-0 flex-col overflow-hidden">
        <div class="border-b px-4 py-2.5" style="border-color: var(--oc-divider-soft);">
          <h3 class="text-base font-semibold" style="color: var(--oc-text-primary);">消息渠道</h3>
          <p class="mt-0.5 text-xs" style="color: var(--oc-text-muted);">在左侧选择渠道，右侧分段菜单切换对应配置表单。</p>
        </div>

        <div class="min-h-0 flex-1 overflow-y-auto p-3">
          <div class="space-y-2">
            <button
              v-for="channel in channelList"
              :key="channel.id"
              type="button"
              class="oc-subpanel w-full border p-3 text-left transition-all duration-200"
              :class="[
                selectedChannelId === channel.id
                  ? 'border-[var(--oc-card-border-strong)]'
                  : 'border-[var(--oc-card-border)]'
              ]"
              :style="{ background: selectedChannelId === channel.id ? 'var(--oc-item-active)' : 'var(--oc-card-elevated)' }"
              @click="selectedChannelId = channel.id"
            >
              <div class="flex items-center gap-3">
                <div class="flex h-9 w-9 items-center justify-center rounded-full border" style="border-color: var(--oc-divider); background: var(--oc-card);">
                  <component :is="channel.icon" class="h-4 w-4" :style="{ color: channel.iconColor }" />
                </div>

                <div class="min-w-0 flex-1">
                  <div class="text-lg font-semibold" style="color: var(--oc-text-primary);">{{ channel.name }}</div>
                  <div class="mt-1 flex items-center gap-2 text-sm">
                    <Check v-if="isConfigured(channel.id)" class="h-3.5 w-3.5" style="color: var(--oc-success);" />
                    <X v-else class="h-3.5 w-3.5" style="color: var(--oc-text-muted);" />
                    <span :style="{ color: isConfigured(channel.id) ? 'var(--oc-success)' : 'var(--oc-text-muted)' }">
                      {{ isConfigured(channel.id) ? '已配置' : '未配置' }}
                    </span>
                    <span style="color: var(--oc-text-quiet);">·</span>
                    <span :style="{ color: forms[channel.id].enabled ? 'var(--oc-success)' : 'var(--oc-text-muted)' }">
                      {{ forms[channel.id].enabled ? '已启用' : '已停用' }}
                    </span>
                  </div>
                </div>

                <ChevronRight class="h-4 w-4" style="color: var(--oc-text-muted);" />
              </div>
            </button>
          </div>
        </div>
      </section>

      <section class="oc-panel flex min-h-0 flex-col overflow-hidden">
        <div class="border-b px-5 py-3" style="border-color: var(--oc-divider-soft);">
          <div class="flex items-start justify-between gap-3">
            <div :class="messageChannelHeaderLayout.leftColumn">
              <div class="flex items-center gap-3">
                <div class="flex h-9 w-9 items-center justify-center rounded-full border" style="border-color: var(--oc-divider); background: var(--oc-card-elevated);">
                  <component :is="selectedChannel.icon" :class="['h-5 w-5', messageChannelHeaderLayout.icon]" :style="{ color: selectedChannel.iconColor }" />
                </div>
                <div :class="messageChannelHeaderLayout.leftMeta">
                  <h3 class="text-xl font-semibold leading-tight" style="color: var(--oc-text-primary);">配置 {{ selectedChannel.name }}</h3>
                  <p class="mt-0.5 text-sm leading-5" style="color: var(--oc-text-muted);">{{ hints[selectedChannelId] }}</p>
                </div>
              </div>
            </div>

            <button
              v-if="isExtensionChannel(selectedChannelId) && !selectedExtensionInstalled"
              class="oc-toolbar-btn h-9 min-w-[104px] px-3 whitespace-nowrap"
              type="button"
              :disabled="installingExtension || extensionStatusLoading"
              @click="openInstallModal"
            >
              <Loader2 v-if="installingExtension && installingChannel === selectedChannelId" class="h-4 w-4 animate-spin" />
              <Download v-else class="h-4 w-4" />
              安装扩展
            </button>
            <div v-else :class="messageChannelHeaderLayout.toggleWrap">
              <span class="text-xs font-medium" style="color: var(--oc-text-muted);">是否启用</span>
              <button
                type="button"
                :class="messageChannelHeaderLayout.toggleControl"
                :style="{
                  borderColor: currentForm.enabled ? 'color-mix(in srgb, var(--oc-success) 55%, transparent)' : 'var(--oc-card-border)',
                  background: currentForm.enabled
                    ? 'color-mix(in srgb, var(--oc-success) 28%, transparent)'
                    : 'color-mix(in srgb, var(--oc-card-elevated) 92%, transparent)'
                }"
                :disabled="!canConfigureCurrentChannel"
                :title="enabledLabel"
                :aria-label="`${selectedChannel.name}${enabledLabel}`"
                :aria-pressed="currentForm.enabled"
                @click="toggleChannelEnabled"
              >
                <span
                  :class="messageChannelHeaderLayout.toggleThumb"
                  :style="{
                    transform: currentForm.enabled ? 'translateX(22px)' : 'translateX(0)'
                  }"
                />
              </button>
              <span class="text-xs" :style="{ color: currentForm.enabled ? 'var(--oc-success)' : 'var(--oc-text-muted)' }">
                {{ enabledLabel }}
              </span>
            </div>
          </div>
        </div>

        <div class="border-b px-4 py-2" style="border-color: var(--oc-divider-soft);">
          <div class="flex flex-wrap items-center gap-2">
            <div v-if="showAccountSelector" class="relative message-channel-account-dropdown">
              <div class="flex items-stretch">
                <Button
                  variant="outline"
                  size="sm"
                  :class="[messageChannelHeaderLayout.accountSelectTrigger, '!rounded-r-none border-r-0 text-sm']"
                  :style="selectedPanel === 'credentials' ? { background: 'var(--oc-item-active)', borderColor: 'var(--oc-card-border-strong)', color: 'var(--oc-text-primary)' } : undefined"
                  :disabled="!canConfigureCurrentChannel || installingExtension"
                  @click.stop="toggleAccountSelectorDropdown"
                >
                  <span class="truncate">{{ currentAccountTabLabel }}</span>
                  <ChevronDown class="h-4 w-4 shrink-0" :class="{ 'rotate-180': showAccountSelectorDropdown }" />
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  class="!rounded-l-none !h-8 px-2.5"
                  :style="selectedPanel === 'credentials' ? { background: 'var(--oc-item-active)', borderColor: 'var(--oc-card-border-strong)', color: 'var(--oc-text-primary)' } : undefined"
                  :disabled="!canConfigureCurrentChannel || installingExtension"
                  @click.stop="openAccountModal"
                >
                  <Plus class="h-3.5 w-3.5" />
                </Button>
              </div>
              <div
                v-if="showAccountSelectorDropdown"
                :class="messageChannelHeaderLayout.accountSelectMenu"
              >
                <button
                  v-for="account in currentAccountOptions"
                  :key="account.id"
                  type="button"
                  class="oc-dropdown-item flex w-full cursor-pointer items-start justify-between gap-3 text-left text-sm"
                  :style="account.id === currentAccountId ? { background: 'var(--oc-item-active)' } : undefined"
                  @click="() => { void handleAccountSelectionChange(account.id) }"
                >
                  <div class="min-w-0">
                    <div class="truncate font-medium" style="color: var(--oc-text-primary);">{{ account.label }}</div>
                    <div class="text-xs" style="color: var(--oc-text-muted);">{{ account.id === 'default' ? '默认账号' : account.id }}</div>
                  </div>
                  <Check v-if="account.id === currentAccountId" class="mt-0.5 h-4 w-4 shrink-0" style="color: var(--oc-accent);" />
                </button>
                <div v-if="currentAccountId !== 'default'" class="oc-dropdown-separator"></div>
                <button
                  v-if="currentAccountId !== 'default'"
                  type="button"
                  class="oc-dropdown-item flex w-full items-center gap-2 text-sm"
                  style="color: var(--oc-danger);"
                  @click="removeSelectedAccount"
                >
                  <Trash2 class="h-3.5 w-3.5" />
                  删除当前账号
                </button>
              </div>
            </div>

            <button
              v-for="tab in publicPanelTabs"
              :key="tab.id"
              type="button"
              class="oc-toolbar-btn h-8 px-3 text-sm"
              :style="tab.id === selectedPanel ? { background: 'var(--oc-item-active)', borderColor: 'var(--oc-card-border-strong)', color: 'var(--oc-text-primary)' } : undefined"
              @click="selectedPanel = tab.id"
            >
              {{ tab.label }}
            </button>
          </div>
          <p class="mt-2 text-xs" style="color: var(--oc-text-muted);">{{ currentPanelScopeHint }}</p>
        </div>

        <div class="min-h-0 flex-1 overflow-y-auto p-5">
          <div
            v-if="selectedNeedsExtensionInstall"
            class="rounded-[12px] border p-4 text-sm"
            style="border-color: var(--oc-card-border); background: var(--oc-card-elevated); color: var(--oc-text-secondary);"
          >
            <p>
              当前渠道 <strong style="color: var(--oc-text-primary);">{{ selectedChannel.name }}</strong> 扩展尚未安装，暂不可配置。
            </p>
            <p class="mt-1">请点击右上角“安装扩展”，安装完成后将自动解锁配置。</p>
          </div>

          <div v-else-if="selectedPanel === 'credentials'" class="space-y-4">
            <template v-if="selectedChannelId === 'telegram' || selectedChannelId === 'discord'">
              <div>
                <label class="mb-1.5 flex items-center gap-2 text-sm font-medium" style="color: var(--oc-text-secondary);">
                  Bot Token（{{ accountScopedFieldPath(selectedChannelId === 'telegram' ? 'botToken' : 'token') }}）<span style="color: var(--oc-danger);">*</span>
                  <Check class="h-3.5 w-3.5" style="color: var(--oc-success);" v-if="currentForm.token" />
                </label>
                <div class="relative">
                  <Input
                    :type="revealToken ? 'text' : 'password'"
                    :model-value="currentForm.token"
                    :placeholder="selectedChannelId === 'telegram' ? '输入 Telegram botToken（由 @BotFather 获取）' : '输入 Discord Bot Token'"
                    class="pr-11"
                    :disabled="!canConfigureCurrentChannel"
                    @update:model-value="(value) => { currentForm.token = value }"
                  />
                  <button
                    type="button"
                    class="absolute inset-y-0 right-0 flex w-10 items-center justify-center transition-colors hover:opacity-80"
                    style="color: var(--oc-text-muted);"
                    :disabled="!canConfigureCurrentChannel"
                    @click="revealToken = !revealToken"
                  >
                    <EyeOff v-if="revealToken" class="h-4 w-4" />
                    <Eye v-else class="h-4 w-4" />
                  </button>
                </div>
              </div>

              <div class="rounded-[12px] border p-3 text-sm" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated); color: var(--oc-text-muted);">
                <template v-if="selectedChannelId === 'telegram'">
                  保存后将写入 <code>{{ accountScopedFieldPath('botToken') }}</code>。
                </template>
                <template v-else>
                  保存后将写入 <code>{{ accountScopedFieldPath('token') }}</code>。
                </template>
              </div>
            </template>

            <template v-else-if="selectedChannelId === 'qq'">
              <div>
                <label class="mb-1.5 flex items-center gap-2 text-sm font-medium" style="color: var(--oc-text-secondary);">
                  {{ `QQ App ID（${accountScopedFieldPath('appId')}）` }}
                  <span style="color: var(--oc-danger);">*</span>
                  <Check class="h-3.5 w-3.5" style="color: var(--oc-success);" v-if="currentForm.token" />
                </label>
                <Input
                  type="text"
                  :model-value="currentForm.token"
                  placeholder="输入 QQ App ID"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.token = value }"
                />
              </div>

              <div>
                <label class="mb-1.5 flex items-center gap-2 text-sm font-medium" style="color: var(--oc-text-secondary);">
                  {{ `QQ App Secret（${accountScopedFieldPath('clientSecret')}）` }}
                  <span style="color: var(--oc-danger);">*</span>
                  <Check class="h-3.5 w-3.5" style="color: var(--oc-success);" v-if="currentForm.userId" />
                </label>
                <div class="relative">
                  <Input
                    :type="revealSecret ? 'text' : 'password'"
                    :model-value="currentForm.userId"
                    placeholder="输入 QQ App Secret"
                    class="pr-11"
                    :disabled="!canConfigureCurrentChannel"
                    @update:model-value="(value) => { currentForm.userId = value }"
                  />
                  <button
                    type="button"
                    class="absolute inset-y-0 right-0 flex w-10 items-center justify-center transition-colors hover:opacity-80"
                    style="color: var(--oc-text-muted);"
                    :disabled="!canConfigureCurrentChannel"
                    @click="revealSecret = !revealSecret"
                  >
                    <EyeOff v-if="revealSecret" class="h-4 w-4" />
                    <Eye v-else class="h-4 w-4" />
                  </button>
                </div>
                <p class="mt-2 text-xs leading-5" style="color: var(--oc-text-muted);">
                  保存后将写入 <code>{{ accountScopedFieldPath('appId') }}</code> 与 <code>{{ accountScopedFieldPath('clientSecret') }}</code>，并清理旧的 <code>token</code> 字段。
                </p>
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">账号名称（{{ accountScopedFieldPath('name') }}）</label>
                <Input
                  :model-value="currentForm.qqName"
                  placeholder="可选，用于区分多个 QQ Bot"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.qqName = value }"
                />
              </div>
            </template>

            <template v-else-if="selectedChannelId === 'feishu' || selectedChannelId === 'wecom' || selectedChannelId === 'dingtalk'">
              <div>
                <label class="mb-1.5 flex items-center gap-2 text-sm font-medium" style="color: var(--oc-text-secondary);">
                  {{ selectedChannelId === 'feishu'
                    ? `飞书 App ID（${accountScopedFieldPath('appId')}）`
                    : selectedChannelId === 'wecom'
                      ? `企业微信 Bot ID（${accountScopedFieldPath('botId')}）`
                      : `Client ID（${accountScopedFieldPath('clientId')}）` }}
                  <span style="color: var(--oc-danger);">*</span>
                  <Check class="h-3.5 w-3.5" style="color: var(--oc-success);" v-if="currentForm.token" />
                </label>
                <Input
                  type="text"
                  :model-value="currentForm.token"
                  :placeholder="selectedChannelId === 'feishu'
                    ? '输入飞书 App ID（如 cli_xxxxx）'
                    : selectedChannelId === 'wecom'
                      ? '输入企业微信 Bot ID'
                      : '输入 Client ID'"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.token = value }"
                />
              </div>

              <div>
                <label class="mb-1.5 flex items-center gap-2 text-sm font-medium" style="color: var(--oc-text-secondary);">
                  {{ selectedChannelId === 'feishu'
                    ? `飞书 App Secret（${accountScopedFieldPath('appSecret')}）`
                    : selectedChannelId === 'wecom'
                      ? `企业微信 Secret（${accountScopedFieldPath('secret')}）`
                      : `Client Secret（${accountScopedFieldPath('clientSecret')}）` }}
                  <span style="color: var(--oc-danger);">*</span>
                  <Check class="h-3.5 w-3.5" style="color: var(--oc-success);" v-if="currentForm.userId" />
                </label>
                <div class="relative">
                  <Input
                    :type="revealSecret ? 'text' : 'password'"
                    :model-value="currentForm.userId"
                    :placeholder="selectedChannelId === 'feishu'
                      ? '输入飞书 App Secret'
                      : selectedChannelId === 'wecom'
                        ? '输入企业微信 Secret'
                        : '输入 Client Secret'"
                    class="pr-11"
                    :disabled="!canConfigureCurrentChannel"
                    @update:model-value="(value) => { currentForm.userId = value }"
                  />
                  <button
                    type="button"
                    class="absolute inset-y-0 right-0 flex w-10 items-center justify-center transition-colors hover:opacity-80"
                    style="color: var(--oc-text-muted);"
                    :disabled="!canConfigureCurrentChannel"
                    @click="revealSecret = !revealSecret"
                  >
                    <EyeOff v-if="revealSecret" class="h-4 w-4" />
                    <Eye v-else class="h-4 w-4" />
                  </button>
                </div>
              </div>

              <template v-if="selectedChannelId === 'wecom'">
                <div>
                  <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">账号名称（{{ accountScopedFieldPath('name') }}）</label>
                  <Input
                    :model-value="currentForm.wecomName"
                    placeholder="可选，用于区分多个企业微信机器人"
                    :disabled="!canConfigureCurrentChannel"
                    @update:model-value="(value) => { currentForm.wecomName = value }"
                  />
                </div>
              </template>

              <template v-else-if="selectedChannelId === 'dingtalk'">
                <div class="grid gap-4 md:grid-cols-2">
                  <div>
                    <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">账号名称（{{ accountScopedFieldPath('name') }}）</label>
                    <Input
                      :model-value="currentForm.dingtalkName"
                      placeholder="可选，用于区分多个账号"
                      :disabled="!canConfigureCurrentChannel"
                      @update:model-value="(value) => { currentForm.dingtalkName = value }"
                    />
                  </div>

                  <div>
                    <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">Agent ID（{{ accountScopedFieldPath('agentId') }}）</label>
                    <Input
                      :model-value="currentForm.dingtalkAgentId"
                      placeholder="main / 123456"
                      :disabled="!canConfigureCurrentChannel"
                      @update:model-value="(value) => { currentForm.dingtalkAgentId = value }"
                    />
                  </div>

                  <div>
                    <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">Robot Code（{{ accountScopedFieldPath('robotCode') }}）</label>
                    <Input
                      :model-value="currentForm.dingtalkRobotCode"
                      placeholder="可选，发送媒体时需要"
                      :disabled="!canConfigureCurrentChannel"
                      @update:model-value="(value) => { currentForm.dingtalkRobotCode = value }"
                    />
                  </div>

                  <div>
                    <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">Corp ID（{{ accountScopedFieldPath('corpId') }}）</label>
                    <Input
                      :model-value="currentForm.dingtalkCorpId"
                      placeholder="可选，部分能力需要"
                      :disabled="!canConfigureCurrentChannel"
                      @update:model-value="(value) => { currentForm.dingtalkCorpId = value }"
                    />
                  </div>
                </div>
              </template>
            </template>

            <template v-else-if="selectedChannelId === 'slack'">
              <div>
                <label class="mb-1.5 flex items-center gap-2 text-sm font-medium" style="color: var(--oc-text-secondary);">
                  Bot Token（{{ accountScopedFieldPath('botToken') }}）<span style="color: var(--oc-danger);">*</span>
                  <Check class="h-3.5 w-3.5" style="color: var(--oc-success);" v-if="currentForm.slackBotToken" />
                </label>
                <div class="relative">
                  <Input
                    :type="revealToken ? 'text' : 'password'"
                    :model-value="currentForm.slackBotToken"
                    placeholder="xoxb-..."
                    class="pr-11"
                    :disabled="!canConfigureCurrentChannel"
                    @update:model-value="(value) => { currentForm.slackBotToken = value }"
                  />
                  <button
                    type="button"
                    class="absolute inset-y-0 right-0 flex w-10 items-center justify-center transition-colors hover:opacity-80"
                    style="color: var(--oc-text-muted);"
                    :disabled="!canConfigureCurrentChannel"
                    @click="revealToken = !revealToken"
                  >
                    <EyeOff v-if="revealToken" class="h-4 w-4" />
                    <Eye v-else class="h-4 w-4" />
                  </button>
                </div>
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">App Token（{{ accountScopedFieldPath('appToken') }}）</label>
                <Input
                  :model-value="currentForm.slackAppToken"
                  :type="revealSecret ? 'text' : 'password'"
                  placeholder="xapp-..."
                  class="pr-11"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.slackAppToken = value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">Signing Secret（{{ accountScopedFieldPath('signingSecret') }}）</label>
                <div class="relative">
                  <Input
                    :type="revealSlackSigningSecret ? 'text' : 'password'"
                    :model-value="currentForm.slackSigningSecret"
                    placeholder="可选（HTTP 模式校验）"
                    class="pr-11"
                    :disabled="!canConfigureCurrentChannel"
                    @update:model-value="(value) => { currentForm.slackSigningSecret = value }"
                  />
                  <button
                    type="button"
                    class="absolute inset-y-0 right-0 flex w-10 items-center justify-center transition-colors hover:opacity-80"
                    style="color: var(--oc-text-muted);"
                    :disabled="!canConfigureCurrentChannel"
                    @click="revealSlackSigningSecret = !revealSlackSigningSecret"
                  >
                    <EyeOff v-if="revealSlackSigningSecret" class="h-4 w-4" />
                    <Eye v-else class="h-4 w-4" />
                  </button>
                </div>
              </div>
            </template>

            <template v-else-if="selectedChannelId === 'whatsapp' || selectedChannelId === 'imessage'">
              <div class="rounded-[12px] border p-3 text-sm" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated); color: var(--oc-text-muted);">
                <template v-if="selectedChannelId === 'whatsapp'">
                  WhatsApp 无需固定 App Token，本渠道主要在“连接模式/访问策略/高级配置”中设置会话与路由参数。
                </template>
                <template v-else>
                  iMessage 无需固定 App Token，本渠道主要在“连接模式/访问策略/高级配置”中设置 rustpush 与路由参数。
                </template>
              </div>
            </template>

            <template v-else>
              <div class="rounded-[12px] border p-3 text-sm" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated); color: var(--oc-text-muted);">
                当前渠道暂无凭据配置项。
              </div>
            </template>
          </div>

          <div v-else-if="selectedPanel === 'access'" class="space-y-4">
            <template v-if="selectedChannelId === 'telegram'">
              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">DM 策略（channels.telegram.dmPolicy）</label>
                <select v-model="currentForm.telegramDmPolicy" class="oc-select" :disabled="!canConfigureCurrentChannel">
                  <option value="pairing">pairing（推荐）</option>
                  <option value="allowlist">allowlist</option>
                  <option value="open">open</option>
                  <option value="disabled">disabled</option>
                </select>
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">DM allowFrom（channels.telegram.allowFrom）</label>
                <textarea
                  class="oc-textarea"
                  :value="currentForm.telegramAllowFrom"
                  placeholder="每行一个 user id，open 模式建议包含 *"
                  :disabled="!canConfigureCurrentChannel"
                  @input="(event) => { currentForm.telegramAllowFrom = (event.target as HTMLTextAreaElement).value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">群组策略（channels.telegram.groupPolicy）</label>
                <select v-model="currentForm.telegramGroupPolicy" class="oc-select" :disabled="!canConfigureCurrentChannel">
                  <option value="allowlist">allowlist（推荐）</option>
                  <option value="open">open</option>
                  <option value="disabled">disabled</option>
                </select>
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">群组发送者白名单（channels.telegram.groupAllowFrom）</label>
                <textarea
                  class="oc-textarea"
                  :value="currentForm.telegramGroupAllowFrom"
                  placeholder="每行一个 Telegram user id"
                  :disabled="!canConfigureCurrentChannel"
                  @input="(event) => { currentForm.telegramGroupAllowFrom = (event.target as HTMLTextAreaElement).value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">群组白名单（channels.telegram.groups）</label>
                <textarea
                  class="oc-textarea"
                  :value="currentForm.telegramGroups"
                  placeholder="每行一个 group/chat id"
                  :disabled="!canConfigureCurrentChannel"
                  @input="(event) => { currentForm.telegramGroups = (event.target as HTMLTextAreaElement).value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">默认投递目标（channels.telegram.defaultTo）</label>
                <Input
                  :model-value="currentForm.telegramDefaultTo"
                  placeholder="可选，CLI --deliver 默认目标"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.telegramDefaultTo = value }"
                />
              </div>
            </template>

            <template v-else-if="selectedChannelId === 'discord'">
              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">DM 策略（channels.discord.dm.policy）</label>
                <select v-model="currentForm.discordDmPolicy" class="oc-select" :disabled="!canConfigureCurrentChannel">
                  <option value="pairing">pairing（推荐）</option>
                  <option value="allowlist">allowlist</option>
                  <option value="open">open</option>
                  <option value="disabled">disabled</option>
                </select>
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">DM allowFrom（channels.discord.dm.allowFrom）</label>
                <textarea
                  class="oc-textarea"
                  :value="currentForm.discordAllowFrom"
                  placeholder="每行一个 user id，open 模式建议包含 *"
                  :disabled="!canConfigureCurrentChannel"
                  @input="(event) => { currentForm.discordAllowFrom = (event.target as HTMLTextAreaElement).value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">群组策略（channels.discord.groupPolicy）</label>
                <select v-model="currentForm.discordGroupPolicy" class="oc-select" :disabled="!canConfigureCurrentChannel">
                  <option value="allowlist">allowlist（推荐）</option>
                  <option value="open">open</option>
                  <option value="disabled">disabled</option>
                </select>
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">Guild/Channel 白名单（channels.discord.guilds）</label>
                <textarea
                  class="oc-textarea"
                  :value="currentForm.discordGuildChannels"
                  placeholder="每行一个 guildId 或 guildId/channelId"
                  :disabled="!canConfigureCurrentChannel"
                  @input="(event) => { currentForm.discordGuildChannels = (event.target as HTMLTextAreaElement).value }"
                />
              </div>
            </template>

            <template v-else-if="selectedChannelId === 'slack'">
              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">DM 策略（channels.slack.dmPolicy）</label>
                <select v-model="currentForm.slackDmPolicy" class="oc-select" :disabled="!canConfigureCurrentChannel">
                  <option value="pairing">pairing（推荐）</option>
                  <option value="allowlist">allowlist</option>
                  <option value="open">open</option>
                  <option value="disabled">disabled</option>
                </select>
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">DM allowFrom（channels.slack.allowFrom）</label>
                <textarea
                  class="oc-textarea"
                  :value="currentForm.slackAllowFrom"
                  placeholder="每行一个 Slack user id，open 模式建议包含 *"
                  :disabled="!canConfigureCurrentChannel"
                  @input="(event) => { currentForm.slackAllowFrom = (event.target as HTMLTextAreaElement).value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">群组策略（channels.slack.groupPolicy）</label>
                <select v-model="currentForm.slackGroupPolicy" class="oc-select" :disabled="!canConfigureCurrentChannel">
                  <option value="allowlist">allowlist（推荐）</option>
                  <option value="open">open</option>
                  <option value="disabled">disabled</option>
                </select>
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">频道白名单（channels.slack.channels）</label>
                <textarea
                  class="oc-textarea"
                  :value="currentForm.slackChannels"
                  placeholder="每行一个 Slack channel id"
                  :disabled="!canConfigureCurrentChannel"
                  @input="(event) => { currentForm.slackChannels = (event.target as HTMLTextAreaElement).value }"
                />
              </div>
            </template>

            <template v-else-if="selectedChannelId === 'whatsapp'">
              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">DM 策略（channels.whatsapp.dmPolicy）</label>
                <select v-model="currentForm.whatsappDmPolicy" class="oc-select" :disabled="!canConfigureCurrentChannel">
                  <option value="pairing">pairing（推荐）</option>
                  <option value="allowlist">allowlist</option>
                  <option value="open">open</option>
                  <option value="disabled">disabled</option>
                </select>
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">DM allowFrom（channels.whatsapp.allowFrom）</label>
                <textarea
                  class="oc-textarea"
                  :value="currentForm.whatsappAllowFrom"
                  placeholder="每行一个号码或 sender id，open 模式建议包含 *"
                  :disabled="!canConfigureCurrentChannel"
                  @input="(event) => { currentForm.whatsappAllowFrom = (event.target as HTMLTextAreaElement).value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">群组策略（channels.whatsapp.groupPolicy）</label>
                <select v-model="currentForm.whatsappGroupPolicy" class="oc-select" :disabled="!canConfigureCurrentChannel">
                  <option value="allowlist">allowlist（推荐）</option>
                  <option value="open">open</option>
                  <option value="disabled">disabled</option>
                </select>
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">群组发送者白名单（channels.whatsapp.groupAllowFrom）</label>
                <textarea
                  class="oc-textarea"
                  :value="currentForm.whatsappGroupAllowFrom"
                  placeholder="每行一个 sender id"
                  :disabled="!canConfigureCurrentChannel"
                  @input="(event) => { currentForm.whatsappGroupAllowFrom = (event.target as HTMLTextAreaElement).value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">群组白名单（channels.whatsapp.groups）</label>
                <textarea
                  class="oc-textarea"
                  :value="currentForm.whatsappGroups"
                  placeholder="每行一个 group id（如 1203630...@g.us）"
                  :disabled="!canConfigureCurrentChannel"
                  @input="(event) => { currentForm.whatsappGroups = (event.target as HTMLTextAreaElement).value }"
                />
              </div>
            </template>

            <template v-else-if="selectedChannelId === 'imessage'">
              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">DM 策略（channels.imessage.dmPolicy）</label>
                <select v-model="currentForm.imessageDmPolicy" class="oc-select" :disabled="!canConfigureCurrentChannel">
                  <option value="pairing">pairing（推荐）</option>
                  <option value="allowlist">allowlist</option>
                  <option value="open">open</option>
                  <option value="disabled">disabled</option>
                </select>
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">DM allowFrom（channels.imessage.allowFrom）</label>
                <textarea
                  class="oc-textarea"
                  :value="currentForm.imessageAllowFrom"
                  placeholder="每行一个 handle（手机号/邮箱）"
                  :disabled="!canConfigureCurrentChannel"
                  @input="(event) => { currentForm.imessageAllowFrom = (event.target as HTMLTextAreaElement).value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">群组策略（channels.imessage.groupPolicy）</label>
                <select v-model="currentForm.imessageGroupPolicy" class="oc-select" :disabled="!canConfigureCurrentChannel">
                  <option value="allowlist">allowlist（推荐）</option>
                  <option value="open">open</option>
                  <option value="disabled">disabled</option>
                </select>
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">群组发送者白名单（channels.imessage.groupAllowFrom）</label>
                <textarea
                  class="oc-textarea"
                  :value="currentForm.imessageGroupAllowFrom"
                  placeholder="每行一个 handle"
                  :disabled="!canConfigureCurrentChannel"
                  @input="(event) => { currentForm.imessageGroupAllowFrom = (event.target as HTMLTextAreaElement).value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">群组白名单（channels.imessage.groups）</label>
                <textarea
                  class="oc-textarea"
                  :value="currentForm.imessageGroups"
                  placeholder="每行一个 group/chat id"
                  :disabled="!canConfigureCurrentChannel"
                  @input="(event) => { currentForm.imessageGroups = (event.target as HTMLTextAreaElement).value }"
                />
              </div>
            </template>

            <template v-if="false">
              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">DM 策略（channels.wecom.dmPolicy）</label>
                <select v-model="currentForm.wecomDmPolicy" class="oc-select" :disabled="!canConfigureCurrentChannel">
                  <option value="pairing">pairing</option>
                  <option value="allowlist">allowlist</option>
                  <option value="open">open</option>
                  <option value="disabled">disabled</option>
                </select>
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">DM allowFrom（channels.wecom.allowFrom）</label>
                <textarea
                  class="oc-textarea"
                  :value="currentForm.wecomAllowFrom"
                  placeholder="每行一个用户 ID，open 模式建议包含 *"
                  :disabled="!canConfigureCurrentChannel"
                  @input="(event) => { currentForm.wecomAllowFrom = (event.target as HTMLTextAreaElement).value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">群组策略（channels.wecom.groupPolicy）</label>
                <select v-model="currentForm.wecomGroupPolicy" class="oc-select" :disabled="!canConfigureCurrentChannel">
                  <option value="open">open</option>
                  <option value="allowlist">allowlist</option>
                  <option value="disabled">disabled</option>
                </select>
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">群发送者白名单（channels.wecom.groupAllowFrom）</label>
                <textarea
                  class="oc-textarea"
                  :value="currentForm.wecomGroupAllowFrom"
                  placeholder="每行一个发送者用户 ID"
                  :disabled="!canConfigureCurrentChannel"
                  @input="(event) => { currentForm.wecomGroupAllowFrom = (event.target as HTMLTextAreaElement).value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">群组白名单（channels.wecom.groups）</label>
                <textarea
                  class="oc-textarea"
                  :value="currentForm.wecomGroups"
                  placeholder="每行一个群组 chatId"
                  :disabled="!canConfigureCurrentChannel"
                  @input="(event) => { currentForm.wecomGroups = (event.target as HTMLTextAreaElement).value }"
                />
              </div>
            </template>


            <template v-else-if="selectedChannelId === 'feishu'">
              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">DM 策略（channels.feishu.dmPolicy）</label>
                <select v-model="currentForm.feishuDmPolicy" class="oc-select" :disabled="!canConfigureCurrentChannel">
                  <option value="pairing">pairing（推荐）</option>
                  <option value="allowlist">allowlist</option>
                  <option value="open">open</option>
                </select>
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">DM allowFrom（channels.feishu.allowFrom）</label>
                <textarea
                  class="oc-textarea"
                  :value="currentForm.feishuAllowFrom"
                  placeholder="每行一个 open_id，open 模式建议包含 *"
                  :disabled="!canConfigureCurrentChannel"
                  @input="(event) => { currentForm.feishuAllowFrom = (event.target as HTMLTextAreaElement).value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">群组策略（channels.feishu.groupPolicy）</label>
                <select v-model="currentForm.feishuGroupPolicy" class="oc-select" :disabled="!canConfigureCurrentChannel">
                  <option value="allowlist">allowlist（推荐）</option>
                  <option value="open">open</option>
                  <option value="disabled">disabled</option>
                </select>
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">群组白名单（channels.feishu.groupAllowFrom）</label>
                <textarea
                  class="oc-textarea"
                  :value="currentForm.feishuGroupAllowFrom"
                  placeholder="每行一个 group id"
                  :disabled="!canConfigureCurrentChannel"
                  @input="(event) => { currentForm.feishuGroupAllowFrom = (event.target as HTMLTextAreaElement).value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">群聊命令绕过策略（channels.feishu.groupCommandMentionBypass）</label>
                <select v-model="currentForm.feishuGroupCommandMentionBypass" class="oc-select" :disabled="!canConfigureCurrentChannel">
                  <option value="single_bot">single_bot（推荐）</option>
                  <option value="never">never</option>
                  <option value="always">always</option>
                </select>
              </div>
            </template>

            <template v-else-if="selectedChannelId === 'dingtalk'">
              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">DM 策略（channels.dingtalk.dmPolicy）</label>
                <select v-model="currentForm.dingtalkDmPolicy" class="oc-select" :disabled="!canConfigureCurrentChannel">
                  <option value="open">open</option>
                  <option value="pairing">pairing</option>
                  <option value="allowlist">allowlist</option>
                </select>
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">allowFrom（channels.dingtalk.allowFrom）</label>
                <textarea
                  class="oc-textarea"
                  :value="currentForm.dingtalkAllowFrom"
                  placeholder="每行一个 sender id"
                  :disabled="!canConfigureCurrentChannel"
                  @input="(event) => { currentForm.dingtalkAllowFrom = (event.target as HTMLTextAreaElement).value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">群组策略（channels.dingtalk.groupPolicy）</label>
                <select v-model="currentForm.dingtalkGroupPolicy" class="oc-select" :disabled="!canConfigureCurrentChannel">
                  <option value="open">open</option>
                  <option value="allowlist">allowlist</option>
                </select>
              </div>
            </template>

            <template v-else>
              <div class="rounded-[12px] border p-3 text-sm" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated); color: var(--oc-text-muted);">
                当前渠道暂无访问策略配置项。
              </div>
            </template>
          </div>

          <div v-else-if="selectedPanel === 'connection'" class="space-y-4">
            <template v-if="selectedChannelId === 'wecom'">
              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">WebSocket URL（channels.wecom.websocketUrl）</label>
                <Input
                  :model-value="currentForm.wecomWebsocketUrl"
                  placeholder="可选，默认使用插件内置地址"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.wecomWebsocketUrl = value }"
                />
              </div>
            </template>

            <template v-else-if="selectedChannelId === 'feishu'">
              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">连接模式（channels.feishu.connectionMode）</label>
                <select v-model="currentForm.feishuConnectionMode" class="oc-select" :disabled="!canConfigureCurrentChannel">
                  <option value="websocket">websocket</option>
                  <option value="webhook">webhook</option>
                </select>
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">域名（channels.feishu.domain）</label>
                <Input
                  :model-value="currentForm.feishuDomain"
                  placeholder="feishu / lark / https://custom-domain"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.feishuDomain = value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">Webhook 路径（channels.feishu.webhookPath）</label>
                <Input
                  :model-value="currentForm.feishuWebhookPath"
                  placeholder="/feishu/events"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.feishuWebhookPath = value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">Webhook 端口（channels.feishu.webhookPort）</label>
                <Input
                  :model-value="currentForm.feishuWebhookPort"
                  placeholder="3000"
                  inputmode="numeric"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.feishuWebhookPort = value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">Encrypt Key（channels.feishu.encryptKey）</label>
                <Input
                  :model-value="currentForm.feishuEncryptKey"
                  placeholder="可选"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.feishuEncryptKey = value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">Verification Token（channels.feishu.verificationToken）</label>
                <Input
                  :model-value="currentForm.feishuVerificationToken"
                  placeholder="可选"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.feishuVerificationToken = value }"
                />
              </div>
            </template>

            <template v-else-if="selectedChannelId === 'slack'">
              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">连接模式（channels.slack.mode）</label>
                <select v-model="currentForm.slackMode" class="oc-select" :disabled="!canConfigureCurrentChannel">
                  <option value="http">http</option>
                  <option value="socket">socket</option>
                </select>
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">Webhook 路径（channels.slack.webhookPath）</label>
                <Input
                  :model-value="currentForm.slackWebhookPath"
                  placeholder="/webhooks/slack"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.slackWebhookPath = value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">Webhook 端口（channels.slack.webhookPort）</label>
                <Input
                  :model-value="currentForm.slackWebhookPort"
                  placeholder="3444"
                  inputmode="numeric"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.slackWebhookPort = value }"
                />
              </div>
            </template>

            <template v-else-if="selectedChannelId === 'whatsapp'">
              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">会话目录（channels.whatsapp.sessionDir）</label>
                <Input
                  :model-value="currentForm.whatsappSessionDir"
                  placeholder="~/.openclaw/.wwebjs_auth"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.whatsappSessionDir = value }"
                />
              </div>

              <label class="inline-flex items-center gap-2 text-sm" style="color: var(--oc-text-secondary);">
                <input
                  type="checkbox"
                  :checked="currentForm.whatsappUseRemoteAuth"
                  :disabled="!canConfigureCurrentChannel"
                  @change="(event) => { currentForm.whatsappUseRemoteAuth = (event.target as HTMLInputElement).checked }"
                />
                启用远程认证（channels.whatsapp.useRemoteAuth）
              </label>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">Webhook 路径（channels.whatsapp.webhookPath）</label>
                <Input
                  :model-value="currentForm.whatsappWebhookPath"
                  placeholder="/webhooks/whatsapp"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.whatsappWebhookPath = value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">Webhook 端口（channels.whatsapp.webhookPort）</label>
                <Input
                  :model-value="currentForm.whatsappWebhookPort"
                  placeholder="3443"
                  inputmode="numeric"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.whatsappWebhookPort = value }"
                />
              </div>
            </template>

            <template v-else-if="selectedChannelId === 'imessage'">
              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">rustpush 路径（channels.imessage.cliPath）</label>
                <Input
                  :model-value="currentForm.imessageCliPath"
                  placeholder="rustpush"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.imessageCliPath = value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">chat.db 路径（channels.imessage.dbPath）</label>
                <Input
                  :model-value="currentForm.imessageDbPath"
                  placeholder="~/Library/Messages/chat.db"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.imessageDbPath = value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">远端主机（channels.imessage.remoteHost）</label>
                <Input
                  :model-value="currentForm.imessageRemoteHost"
                  placeholder="可选：iMessage 网关地址"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.imessageRemoteHost = value }"
                />
              </div>
            </template>

            <template v-else>
              <div class="rounded-[12px] border p-3 text-sm" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated); color: var(--oc-text-muted);">
                当前渠道暂无连接模式配置项。
              </div>
            </template>
          </div>

          <div v-else-if="false" class="space-y-4">
            <div class="rounded-[12px] border p-3 text-sm" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated); color: var(--oc-text-muted);">
              当前版本的钉钉插件使用 Stream 模式，不再需要单独的网关配置。
            </div>
          </div>

          <div v-else-if="selectedPanel === 'advanced'" class="space-y-4">
            <template v-if="selectedChannelId === 'telegram'">
              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">ReplyTo 模式（channels.telegram.replyToMode）</label>
                <select v-model="currentForm.telegramReplyToMode" class="oc-select" :disabled="!canConfigureCurrentChannel">
                  <option value="off">off</option>
                  <option value="first">first</option>
                  <option value="all">all</option>
                </select>
              </div>
            </template>

            <template v-else-if="selectedChannelId === 'discord'">
              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">ReplyTo 模式（channels.discord.replyToMode）</label>
                <select v-model="currentForm.discordReplyToMode" class="oc-select" :disabled="!canConfigureCurrentChannel">
                  <option value="off">off</option>
                  <option value="first">first</option>
                  <option value="all">all</option>
                </select>
              </div>
            </template>

            <template v-else-if="selectedChannelId === 'slack'">
              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">ReplyTo 模式（channels.slack.replyToMode）</label>
                <select v-model="currentForm.slackReplyToMode" class="oc-select" :disabled="!canConfigureCurrentChannel">
                  <option value="off">off</option>
                  <option value="first">first</option>
                  <option value="all">all</option>
                </select>
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">默认投递目标（channels.slack.defaultTo）</label>
                <Input
                  :model-value="currentForm.slackDefaultTo"
                  placeholder="可选：默认 DM/Channel"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.slackDefaultTo = value }"
                />
              </div>

              <label class="inline-flex items-center gap-2 text-sm" style="color: var(--oc-text-secondary);">
                <input
                  type="checkbox"
                  :checked="currentForm.slackRequireMention"
                  :disabled="!canConfigureCurrentChannel"
                  @change="(event) => { currentForm.slackRequireMention = (event.target as HTMLInputElement).checked }"
                />
                仅处理 @mention（channels.slack.requireMention）
              </label>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">文本分块阈值（channels.slack.textChunkLimit）</label>
                <Input
                  :model-value="currentForm.slackTextChunkLimit"
                  placeholder="可选，正整数"
                  inputmode="numeric"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.slackTextChunkLimit = value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">分块模式（channels.slack.chunkMode）</label>
                <Input
                  :model-value="currentForm.slackChunkMode"
                  placeholder="sentence"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.slackChunkMode = value }"
                />
              </div>
            </template>

            <template v-else-if="selectedChannelId === 'whatsapp'">
              <label class="inline-flex items-center gap-2 text-sm" style="color: var(--oc-text-secondary);">
                <input
                  type="checkbox"
                  :checked="currentForm.whatsappIncludeAttachments"
                  :disabled="!canConfigureCurrentChannel"
                  @change="(event) => { currentForm.whatsappIncludeAttachments = (event.target as HTMLInputElement).checked }"
                />
                处理附件消息（channels.whatsapp.includeAttachments）
              </label>

              <label class="inline-flex items-center gap-2 text-sm" style="color: var(--oc-text-secondary);">
                <input
                  type="checkbox"
                  :checked="currentForm.whatsappSendReadReceipts"
                  :disabled="!canConfigureCurrentChannel"
                  @change="(event) => { currentForm.whatsappSendReadReceipts = (event.target as HTMLInputElement).checked }"
                />
                发送已读回执（channels.whatsapp.sendReadReceipts）
              </label>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">媒体大小上限 MB（channels.whatsapp.mediaMaxMb）</label>
                <Input
                  :model-value="currentForm.whatsappMediaMaxMb"
                  placeholder="30"
                  inputmode="decimal"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.whatsappMediaMaxMb = value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">文本分块阈值（channels.whatsapp.textChunkLimit）</label>
                <Input
                  :model-value="currentForm.whatsappTextChunkLimit"
                  placeholder="可选，正整数"
                  inputmode="numeric"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.whatsappTextChunkLimit = value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">分块模式（channels.whatsapp.chunkMode）</label>
                <Input
                  :model-value="currentForm.whatsappChunkMode"
                  placeholder="sentence"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.whatsappChunkMode = value }"
                />
              </div>
            </template>

            <template v-else-if="selectedChannelId === 'imessage'">
              <label class="inline-flex items-center gap-2 text-sm" style="color: var(--oc-text-secondary);">
                <input
                  type="checkbox"
                  :checked="currentForm.imessageIncludeAttachments"
                  :disabled="!canConfigureCurrentChannel"
                  @change="(event) => { currentForm.imessageIncludeAttachments = (event.target as HTMLInputElement).checked }"
                />
                处理附件消息（channels.imessage.includeAttachments）
              </label>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">附件根目录（channels.imessage.attachmentRoots）</label>
                <textarea
                  class="oc-textarea"
                  :value="currentForm.imessageAttachmentRoots"
                  placeholder="每行一个本地目录"
                  :disabled="!canConfigureCurrentChannel"
                  @input="(event) => { currentForm.imessageAttachmentRoots = (event.target as HTMLTextAreaElement).value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">远端附件根目录（channels.imessage.remoteAttachmentRoots）</label>
                <textarea
                  class="oc-textarea"
                  :value="currentForm.imessageRemoteAttachmentRoots"
                  placeholder="每行一个远端目录"
                  :disabled="!canConfigureCurrentChannel"
                  @input="(event) => { currentForm.imessageRemoteAttachmentRoots = (event.target as HTMLTextAreaElement).value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">媒体大小上限 MB（channels.imessage.mediaMaxMb）</label>
                <Input
                  :model-value="currentForm.imessageMediaMaxMb"
                  placeholder="30"
                  inputmode="decimal"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.imessageMediaMaxMb = value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">服务类型（channels.imessage.service）</label>
                <Input
                  :model-value="currentForm.imessageService"
                  placeholder="auto / iMessage / SMS"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.imessageService = value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">区域（channels.imessage.region）</label>
                <Input
                  :model-value="currentForm.imessageRegion"
                  placeholder="CN"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.imessageRegion = value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">文本分块阈值（channels.imessage.textChunkLimit）</label>
                <Input
                  :model-value="currentForm.imessageTextChunkLimit"
                  placeholder="可选，正整数"
                  inputmode="numeric"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.imessageTextChunkLimit = value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">分块模式（channels.imessage.chunkMode）</label>
                <Input
                  :model-value="currentForm.imessageChunkMode"
                  placeholder="sentence"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.imessageChunkMode = value }"
                />
              </div>
            </template>

            <template v-else-if="selectedChannelId === 'wecom'">
              <label class="inline-flex items-center gap-2 text-sm" style="color: var(--oc-text-secondary);">
                <input
                  type="checkbox"
                  :checked="currentForm.wecomSendThinkingMessage"
                  :disabled="!canConfigureCurrentChannel"
                  @change="(event) => { currentForm.wecomSendThinkingMessage = (event.target as HTMLInputElement).checked }"
                />
                发送思考中提示（channels.wecom.sendThinkingMessage）
              </label>
            </template>

            <template v-else-if="selectedChannelId === 'qq'">
              <label class="inline-flex items-center gap-2 text-sm" style="color: var(--oc-text-secondary);">
                <input
                  type="checkbox"
                  :checked="currentForm.qqMarkdownSupport"
                  :disabled="!canConfigureCurrentChannel"
                  @change="(event) => { currentForm.qqMarkdownSupport = (event.target as HTMLInputElement).checked }"
                />
                启用 Markdown 消息（channels.qqbot.markdownSupport）
              </label>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">系统提示词（channels.qqbot.systemPrompt）</label>
                <textarea
                  class="oc-textarea"
                  :value="currentForm.qqSystemPrompt"
                  placeholder="可选，会拼接到用户消息前"
                  :disabled="!canConfigureCurrentChannel"
                  @input="(event) => { currentForm.qqSystemPrompt = (event.target as HTMLTextAreaElement).value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">图床地址（channels.qqbot.imageServerBaseUrl）</label>
                <Input
                  :model-value="currentForm.qqImageServerBaseUrl"
                  placeholder="http://your-ip:18765"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.qqImageServerBaseUrl = value }"
                />
              </div>
            </template>


            <template v-else-if="selectedChannelId === 'feishu'">
              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">渲染模式（channels.feishu.renderMode）</label>
                <select v-model="currentForm.feishuRenderMode" class="oc-select" :disabled="!canConfigureCurrentChannel">
                  <option value="auto">auto</option>
                  <option value="raw">raw</option>
                  <option value="card">card</option>
                </select>
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">媒体大小上限 MB（channels.feishu.mediaMaxMb）</label>
                <Input
                  :model-value="currentForm.feishuMediaMaxMb"
                  placeholder="30"
                  inputmode="decimal"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.feishuMediaMaxMb = value }"
                />
              </div>

              <label class="inline-flex items-center gap-2 text-sm" style="color: var(--oc-text-secondary);">
                <input
                  type="checkbox"
                  :checked="currentForm.feishuDynamicEnabled"
                  :disabled="!canConfigureCurrentChannel"
                  @change="(event) => { currentForm.feishuDynamicEnabled = (event.target as HTMLInputElement).checked }"
                />
                启用动态 Agent（channels.feishu.dynamicAgentCreation.enabled）
              </label>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">工作区模板（dynamicAgentCreation.workspaceTemplate）</label>
                <Input
                  :model-value="currentForm.feishuDynamicWorkspaceTemplate"
                  placeholder="可选"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.feishuDynamicWorkspaceTemplate = value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">Agent 目录模板（dynamicAgentCreation.agentDirTemplate）</label>
                <Input
                  :model-value="currentForm.feishuDynamicAgentDirTemplate"
                  placeholder="可选"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.feishuDynamicAgentDirTemplate = value }"
                />
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">最大 Agent 数（dynamicAgentCreation.maxAgents）</label>
                <Input
                  :model-value="currentForm.feishuDynamicMaxAgents"
                  placeholder="可选，正整数"
                  inputmode="numeric"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.feishuDynamicMaxAgents = value }"
                />
              </div>
            </template>

            <template v-else-if="selectedChannelId === 'dingtalk'">
              <label class="inline-flex items-center gap-2 text-sm" style="color: var(--oc-text-secondary);">
                <input
                  type="checkbox"
                  :checked="currentForm.dingtalkShowThinking"
                  :disabled="!canConfigureCurrentChannel"
                  @change="(event) => { currentForm.dingtalkShowThinking = (event.target as HTMLInputElement).checked }"
                />
                显示思考中状态（channels.dingtalk.showThinking）
              </label>

              <label class="inline-flex items-center gap-2 text-sm" style="color: var(--oc-text-secondary);">
                <input
                  type="checkbox"
                  :checked="currentForm.dingtalkDebug"
                  :disabled="!canConfigureCurrentChannel"
                  @change="(event) => { currentForm.dingtalkDebug = (event.target as HTMLInputElement).checked }"
                />
                调试日志（channels.dingtalk.debug）
              </label>

              <div class="grid gap-4 md:grid-cols-2">
                <div>
                  <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">回复类型（channels.dingtalk.messageType）</label>
                  <select v-model="currentForm.dingtalkMessageType" class="oc-select" :disabled="!canConfigureCurrentChannel">
                    <option value="markdown">markdown</option>
                    <option value="card">card</option>
                  </select>
                </div>

                <div>
                  <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">卡片模板 Key（channels.dingtalk.cardTemplateKey）</label>
                  <Input
                    :model-value="currentForm.dingtalkCardTemplateKey"
                    placeholder="content"
                    :disabled="!canConfigureCurrentChannel"
                    @update:model-value="(value) => { currentForm.dingtalkCardTemplateKey = value }"
                  />
                </div>

                <div>
                  <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">卡片模板 ID（channels.dingtalk.cardTemplateId）</label>
                  <Input
                    :model-value="currentForm.dingtalkCardTemplateId"
                    placeholder="messageType=card 时可选"
                    :disabled="!canConfigureCurrentChannel"
                    @update:model-value="(value) => { currentForm.dingtalkCardTemplateId = value }"
                  />
                </div>

                <div>
                  <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">媒体大小上限 MB（channels.dingtalk.mediaMaxMb）</label>
                  <Input
                    :model-value="currentForm.dingtalkMediaMaxMb"
                    placeholder="可选，如 30"
                    inputmode="decimal"
                    :disabled="!canConfigureCurrentChannel"
                    @update:model-value="(value) => { currentForm.dingtalkMediaMaxMb = value }"
                  />
                </div>
              </div>

              <div>
                <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">媒体 URL 白名单（channels.dingtalk.mediaUrlAllowlist）</label>
                <textarea
                  class="oc-textarea"
                  :value="currentForm.dingtalkMediaUrlAllowlist"
                  placeholder="每行一个域名或 URL 前缀"
                  :disabled="!canConfigureCurrentChannel"
                  @input="(event) => { currentForm.dingtalkMediaUrlAllowlist = (event.target as HTMLTextAreaElement).value }"
                />
              </div>
            </template>

            <template v-else>
              <div class="rounded-[12px] border p-3 text-sm" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated); color: var(--oc-text-muted);">
                当前渠道暂无高级配置项。
              </div>
            </template>
          </div>
        </div>

        <div class="border-t px-5 py-3 shrink-0" style="border-color: var(--oc-divider-soft);">
          <div class="flex items-center gap-3">
            <Button class="min-w-[132px]" :disabled="!canConfigureCurrentChannel || installingExtension" @click="saveConfig">
              <Save class="h-4 w-4" />
              {{ saveButtonLabel }}
            </Button>
            <Button
              v-if="selectedChannelId === 'feishu'"
              variant="outline"
              class="min-w-[132px] whitespace-nowrap"
              :disabled="!selectedExtensionInstalled || installingExtension || approvingPairing"
              @click="openPairingModal"
            >
              <KeyRound class="h-4 w-4" />
              填写配对码
            </Button>
            <Button
              v-if="selectedChannelId === 'feishu' || selectedChannelId === 'dingtalk'"
              variant="outline"
              class="min-w-[132px] whitespace-nowrap"
              @click="openAppCenter"
            >
              <ExternalLink class="h-4 w-4" />
              申请应用
            </Button>
          </div>
        </div>
      </section>
    </div>

    <CommonInputConfirmModal
      v-if="showPairingCodeModal"
      title="填写配对码"
      description="请填写配对码，或直接粘贴完整命令。"
      :placeholder="FEISHU_PAIRING_APPROVE_COMMAND_EXAMPLE"
      note="支持粘贴完整命令（含 --notify），系统会自动提取配对码。"
      :model-value="pairingInput"
      :loading="approvingPairing"
      :confirm-text="approvingPairing ? '配对中...' : '确认配对'"
      @update:model-value="handlePairingInputChange"
      @cancel="closePairingModal"
      @confirm="submitPairing"
    />

    <CommonInputConfirmModal
      v-if="showAccountModal"
      title="新增账号"
      description="请输入账号 ID。该值会作为 channels.*.accounts 下的键名。"
      placeholder="main / ops / coding-plan"
      note="仅支持字母、数字、点、下划线和中划线。创建后会复制当前账号配置到新账号。"
      :model-value="accountInput"
      :loading="submittingAccount"
      :confirm-text="submittingAccount ? '创建中...' : '创建账号'"
      @update:model-value="handleAccountInputChange"
      @cancel="closeAccountModal"
      @confirm="submitAccount"
    />

    <div v-if="showInstallModal" class="oc-modal-overlay" @click.self="closeInstallModal">
      <Card :class="messageChannelInstallModalLayout.card">
        <div class="flex items-center justify-between gap-2">
          <h3 class="text-lg font-semibold" style="color: var(--oc-text-primary);">
            安装扩展 {{ installingChannelName || selectedChannel.name }}
          </h3>
          <span
            class="inline-flex items-center rounded-[10px] border px-2 py-1 text-xs"
            :style="{
              borderColor: 'var(--oc-card-border)',
              background: 'var(--oc-card-elevated)',
              color: installingExtension ? 'var(--oc-warning)' : 'var(--oc-success)'
            }"
          >
            {{ installingExtension ? '安装中' : '已结束' }}
          </span>
        </div>

        <div :class="messageChannelInstallModalLayout.logViewport">
          <TerminalLog :logs="installLogs" />
        </div>

        <div class="mt-4 flex justify-end gap-2">
          <Button variant="outline" :disabled="installingExtension" @click="closeInstallModal">
            关闭
          </Button>
          <Button
            v-if="!installingExtension && isExtensionChannel(selectedChannelId) && !selectedExtensionInstalled"
            @click="openInstallModal"
          >
            重试安装
          </Button>
        </div>
      </Card>
    </div>
  </div>
</template>
