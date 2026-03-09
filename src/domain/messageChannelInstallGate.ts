import { getChannelPluginMeta } from './channelPluginCatalog'

export type MessageChannelInstallGateInput = {
  channelId: string
  isExtensionChannel: boolean
  installed: boolean
}

export const canConfigureMessageChannelBeforeInstall = (channelId: string): boolean => {
  const meta = getChannelPluginMeta(channelId)
  if (!meta?.needsPluginInstall) return true
  return false
}

export const shouldBlockMessageChannelConfigUntilInstall = ({
  channelId,
  isExtensionChannel,
  installed,
}: MessageChannelInstallGateInput): boolean =>
  isExtensionChannel && !installed && !canConfigureMessageChannelBeforeInstall(channelId)
