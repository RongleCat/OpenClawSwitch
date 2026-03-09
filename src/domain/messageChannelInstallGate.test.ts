import { describe, expect, it } from 'vitest'
import {
  canConfigureMessageChannelBeforeInstall,
  shouldBlockMessageChannelConfigUntilInstall,
} from './messageChannelInstallGate'

describe('canConfigureMessageChannelBeforeInstall', () => {
  it('keeps Feishu gated behind extension installation', () => {
    expect(canConfigureMessageChannelBeforeInstall('feishu')).toBe(false)
  })

  it('keeps Dingtalk gated behind extension installation', () => {
    expect(canConfigureMessageChannelBeforeInstall('dingtalk')).toBe(false)
  })

  it('keeps WeCom gated behind extension installation', () => {
    expect(canConfigureMessageChannelBeforeInstall('wecom')).toBe(false)
  })

  it('keeps QQ gated behind extension installation', () => {
    expect(canConfigureMessageChannelBeforeInstall('qq')).toBe(false)
  })
})

describe('shouldBlockMessageChannelConfigUntilInstall', () => {
  it('blocks Feishu until install completes', () => {
    expect(
      shouldBlockMessageChannelConfigUntilInstall({
        channelId: 'feishu',
        isExtensionChannel: true,
        installed: false,
      })
    ).toBe(true)
  })

  it('still blocks Dingtalk until install completes', () => {
    expect(
      shouldBlockMessageChannelConfigUntilInstall({
        channelId: 'dingtalk',
        isExtensionChannel: true,
        installed: false,
      })
    ).toBe(true)
  })

  it('blocks WeCom until install completes', () => {
    expect(
      shouldBlockMessageChannelConfigUntilInstall({
        channelId: 'wecom',
        isExtensionChannel: true,
        installed: false,
      })
    ).toBe(true)
  })

  it('blocks QQ until install completes', () => {
    expect(
      shouldBlockMessageChannelConfigUntilInstall({
        channelId: 'qq',
        isExtensionChannel: true,
        installed: false,
      })
    ).toBe(true)
  })
})
