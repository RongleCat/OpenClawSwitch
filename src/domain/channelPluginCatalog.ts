export type PluginInstallChannelId = 'feishu' | 'wecom' | 'qq' | 'dingtalk'
export type MessageChannelId =
  | PluginInstallChannelId
  | 'telegram'
  | 'discord'
  | 'slack'
  | 'whatsapp'
  | 'imessage'

export interface ChannelPluginMeta {
  id: MessageChannelId
  configKey: string
  displayName: string
  npmPackage: string | null
  pluginStatusKey: ChannelPluginStatusKey | null
  supportsQuickSetup: boolean
  supportsMessagePage: boolean
  needsPluginInstall: boolean
}

export type ChannelPluginStatusKey =
  | 'feishuInstalled'
  | 'wecomInstalled'
  | 'qqInstalled'
  | 'dingtalkInstalled'

export interface ChannelPluginStatus {
  feishuInstalled: boolean
  wecomInstalled: boolean
  qqInstalled: boolean
  dingtalkInstalled: boolean
}

export const CHANNEL_PLUGIN_CATALOG: ChannelPluginMeta[] = [
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
]

export const getChannelPluginMeta = (channelId: MessageChannelId | string) =>
  CHANNEL_PLUGIN_CATALOG.find((item) => item.id === channelId)

export const getChannelConfigKey = (channelId: MessageChannelId | string) =>
  getChannelPluginMeta(channelId)?.configKey ?? channelId

export const isChannelPluginInstalled = (
  status: ChannelPluginStatus,
  channelId: MessageChannelId | string
) => {
  const statusKey = getChannelPluginMeta(channelId)?.pluginStatusKey
  if (!statusKey) return true
  return Boolean(status[statusKey])
}

export const QUICK_SETUP_CHANNEL_ORDER: PluginInstallChannelId[] = ['feishu', 'wecom', 'qq', 'dingtalk']
export const MESSAGE_CHANNEL_PRIMARY_ORDER: PluginInstallChannelId[] = ['feishu', 'wecom', 'qq', 'dingtalk']
export const PLUGIN_INSTALL_CHANNEL_IDS: PluginInstallChannelId[] = ['feishu', 'wecom', 'qq', 'dingtalk']
