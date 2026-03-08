export type MessageChannelInstallGateInput = {
  channelId: string
  isExtensionChannel: boolean
  installed: boolean
}

export const canConfigureMessageChannelBeforeInstall = (_channelId: string): boolean => false

export const shouldBlockMessageChannelConfigUntilInstall = ({
  channelId,
  isExtensionChannel,
  installed,
}: MessageChannelInstallGateInput): boolean =>
  isExtensionChannel && !installed && !canConfigureMessageChannelBeforeInstall(channelId)
