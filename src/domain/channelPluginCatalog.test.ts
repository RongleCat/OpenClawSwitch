import { describe, expect, it } from 'vitest'
import {
  CHANNEL_PLUGIN_CATALOG,
  MESSAGE_CHANNEL_DISPLAY_ORDER,
  MESSAGE_CHANNEL_PRIMARY_ORDER,
  PLUGIN_INSTALL_CHANNEL_IDS,
  QUICK_SETUP_CHANNEL_ORDER,
  getChannelConfigKey,
  isChannelPluginInstalled,
  getChannelPluginMeta,
  sortMessageChannelsForDisplay,
} from './channelPluginCatalog'

describe('CHANNEL_PLUGIN_CATALOG', () => {
  it('pins the requested npm package names for plugin-backed channels', () => {
    expect(getChannelPluginMeta('feishu')?.npmPackage).toBe('@larksuiteoapi/feishu-openclaw-plugin')
    expect(getChannelPluginMeta('wecom')?.npmPackage).toBe('@wecom/wecom-openclaw-plugin')
    expect(getChannelPluginMeta('qq')?.npmPackage).toBe('@sliverp/qqbot')
    expect(getChannelPluginMeta('dingtalk')?.npmPackage).toBe('@dingtalk-real-ai/dingtalk-connector')
  })

  it('keeps plugin-backed channels installable in the requested order', () => {
    expect(PLUGIN_INSTALL_CHANNEL_IDS).toEqual(['feishu', 'wecom', 'qq', 'dingtalk'])
    expect(QUICK_SETUP_CHANNEL_ORDER).toEqual(['feishu', 'wecom', 'qq', 'dingtalk'])
    expect(MESSAGE_CHANNEL_PRIMARY_ORDER).toEqual(['feishu', 'wecom', 'qq', 'dingtalk'])
  })

  it('exposes distinct config keys for plugin-backed channels', () => {
    expect(getChannelPluginMeta('feishu')?.configKey).toBe('feishu')
    expect(getChannelPluginMeta('wecom')?.configKey).toBe('wecom')
    expect(getChannelPluginMeta('qq')?.configKey).toBe('qqbot')
    expect(getChannelPluginMeta('dingtalk')?.configKey).toBe('dingtalk')
  })

  it('resolves config keys and install status from shared metadata helpers', () => {
    expect(getChannelConfigKey('qq')).toBe('qqbot')
    expect(getChannelConfigKey('wecom')).toBe('wecom')
    expect(getChannelConfigKey('telegram')).toBe('telegram')
    expect(
      isChannelPluginInstalled(
        {
          feishuInstalled: false,
          wecomInstalled: true,
          qqInstalled: false,
          dingtalkInstalled: true,
        },
        'wecom'
      )
    ).toBe(true)
    expect(
      isChannelPluginInstalled(
        {
          feishuInstalled: false,
          wecomInstalled: true,
          qqInstalled: false,
          dingtalkInstalled: true,
        },
        'telegram'
      )
    ).toBe(true)
    expect(
      isChannelPluginInstalled(
        {
          feishuInstalled: false,
          wecomInstalled: true,
          qqInstalled: false,
          dingtalkInstalled: true,
        },
        'qq'
      )
    ).toBe(false)
  })

  it('keeps a stable catalog for all known channels', () => {
    expect(CHANNEL_PLUGIN_CATALOG.map((item) => item.id)).toEqual([
      'feishu',
      'wecom',
      'qq',
      'dingtalk',
      'telegram',
      'discord',
      'slack',
      'whatsapp',
      'imessage',
    ])
  })

  it('pushes telegram and discord to the tail of the message channel display order', () => {
    expect(MESSAGE_CHANNEL_DISPLAY_ORDER.slice(-2)).toEqual(['telegram', 'discord'])

    expect(
      sortMessageChannelsForDisplay([
        { id: 'discord', label: 'Discord' },
        { id: 'slack', label: 'Slack' },
        { id: 'telegram', label: 'Telegram' },
        { id: 'feishu', label: '飞书' },
      ]).map((item) => item.id)
    ).toEqual(['feishu', 'slack', 'telegram', 'discord'])
  })
})
