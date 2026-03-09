import { describe, expect, it } from 'vitest'
import {
  FEISHU_PLUGIN_ALLOW_ENTRY,
  ensureFeishuPluginAllowed,
  mergeFeishuChannelConfig,
} from './feishuPlugin'

describe('ensureFeishuPluginAllowed', () => {
  it('enables plugins and appends feishu plugin once while preserving existing allow entries', () => {
    const root: Record<string, unknown> = {
      plugins: {
        allow: ['telegram', 'custom-plugin'],
      },
    }

    ensureFeishuPluginAllowed(root)
    ensureFeishuPluginAllowed(root)

    expect(root).toEqual({
      plugins: {
        enabled: true,
        allow: ['telegram', 'custom-plugin', FEISHU_PLUGIN_ALLOW_ENTRY],
      },
    })
  })
})

describe('mergeFeishuChannelConfig', () => {
  it('merges feishu credentials and default transport settings into existing config', () => {
    const root: Record<string, unknown> = {
      plugins: {
        allow: ['telegram'],
      },
      channels: {
        feishu: {
          dmPolicy: 'pairing',
        },
      },
    }

    mergeFeishuChannelConfig(root, {
      enabled: true,
      appId: 'cli_a926f047d4a1dcbd',
      appSecret: 'G6ooBg0W32TTDnzhGvEuTDSgGRzNvukA',
      domain: 'feishu',
      connectionMode: 'websocket',
    })

    expect(root).toEqual({
      plugins: {
        enabled: true,
        allow: ['telegram', FEISHU_PLUGIN_ALLOW_ENTRY],
      },
      channels: {
        feishu: {
          dmPolicy: 'pairing',
          enabled: true,
          appId: 'cli_a926f047d4a1dcbd',
          appSecret: 'G6ooBg0W32TTDnzhGvEuTDSgGRzNvukA',
          domain: 'feishu',
          connectionMode: 'websocket',
        },
      },
    })
  })

  it('does not overwrite existing domain or connection mode when optional transport fields are omitted', () => {
    const root: Record<string, unknown> = {
      channels: {
        feishu: {
          domain: 'lark',
          connectionMode: 'webhook',
          verificationToken: 'keep-me',
        },
      },
    }

    mergeFeishuChannelConfig(root, {
      enabled: false,
      appId: 'cli_next',
      appSecret: 'secret_next',
    })

    expect(root).toEqual({
      plugins: {
        enabled: true,
        allow: [FEISHU_PLUGIN_ALLOW_ENTRY],
      },
      channels: {
        feishu: {
          domain: 'lark',
          connectionMode: 'webhook',
          verificationToken: 'keep-me',
          enabled: false,
          appId: 'cli_next',
          appSecret: 'secret_next',
        },
      },
    })
  })
})
