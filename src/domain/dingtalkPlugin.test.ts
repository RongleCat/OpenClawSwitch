import { describe, expect, it } from 'vitest'
import {
  DINGTALK_CHANNEL_BINDING_KEYS,
  DINGTALK_CHANNEL_KEY,
  ensureDingtalkChannelConfigNode,
  ensureDingtalkPluginAllowed,
  mergeDingtalkEditableConfig,
  resolveDingtalkChannelNode,
  shouldIncludeDingtalkDefaultAccount,
} from './dingtalkPlugin'

describe('resolveDingtalkChannelNode', () => {
  it('prefers the new dingtalk channel key', () => {
    expect(
      resolveDingtalkChannelNode({
        dingtalk: { enabled: true, clientId: 'new' },
        'dingtalk-connector': { enabled: true, clientId: 'legacy' },
      })
    ).toEqual({ enabled: true, clientId: 'new' })
  })

  it('falls back to the legacy dingtalk connector key', () => {
    expect(
      resolveDingtalkChannelNode({
        'dingtalk-connector': { enabled: true, clientId: 'legacy' },
      })
    ).toEqual({ enabled: true, clientId: 'legacy' })
  })
})

describe('ensureDingtalkPluginAllowed', () => {
  it('enables plugins and appends dingtalk once', () => {
    const root: Record<string, unknown> = {
      plugins: {
        allow: ['telegram'],
      },
    }

    ensureDingtalkPluginAllowed(root)
    ensureDingtalkPluginAllowed(root)

    expect(root).toEqual({
      plugins: {
        enabled: true,
        allow: ['telegram', 'dingtalk'],
      },
    })
  })
})

describe('shouldIncludeDingtalkDefaultAccount', () => {
  it('includes default account when shared config exists', () => {
    expect(
      shouldIncludeDingtalkDefaultAccount({
        channelNode: { clientId: 'shared' },
        hasMultipleAgents: false,
      })
    ).toBe(true)
  })

  it('includes default account when accounts or multiple agents exist', () => {
    expect(
      shouldIncludeDingtalkDefaultAccount({
        channelNode: { accounts: { main: { clientId: 'main' } } },
        hasMultipleAgents: false,
      })
    ).toBe(true)

    expect(
      shouldIncludeDingtalkDefaultAccount({
        channelNode: {},
        hasMultipleAgents: true,
      })
    ).toBe(true)
  })

  it('exports the new dingtalk keys', () => {
    expect(DINGTALK_CHANNEL_KEY).toBe('dingtalk')
    expect(DINGTALK_CHANNEL_BINDING_KEYS).toEqual(['dingtalk', 'dingtalk-connector'])
  })
})

describe('mergeDingtalkEditableConfig', () => {
  it('preserves unknown plugin fields while replacing editable fields', () => {
    expect(
      mergeDingtalkEditableConfig(
        {
          clientId: 'old',
          gatewayToken: 'legacy',
          showThinking: false,
          mediaMaxMb: 20,
        },
        {
          clientId: 'new',
          debug: true,
        }
      )
    ).toEqual({
      clientId: 'new',
      debug: true,
      showThinking: false,
      mediaMaxMb: 20,
    })
  })
})

describe('ensureDingtalkChannelConfigNode', () => {
  it('migrates legacy dingtalk-connector config into channels.dingtalk', () => {
    const channels: Record<string, unknown> = {
      'dingtalk-connector': {
        clientId: 'legacy-id',
        accounts: {
          ops: { clientId: 'ops-id' },
        },
      },
    }

    const resolved = ensureDingtalkChannelConfigNode(channels)

    expect(resolved).toEqual({
      clientId: 'legacy-id',
      accounts: {
        ops: { clientId: 'ops-id' },
      },
    })
    expect(channels).toEqual({
      dingtalk: {
        clientId: 'legacy-id',
        accounts: {
          ops: { clientId: 'ops-id' },
        },
      },
    })
  })

  it('merges legacy accounts into the new dingtalk key without overwriting new values', () => {
    const channels: Record<string, unknown> = {
      dingtalk: {
        clientId: 'new-id',
        accounts: {
          main: { clientId: 'main-id' },
        },
      },
      'dingtalk-connector': {
        clientSecret: 'legacy-secret',
        accounts: {
          ops: { clientId: 'ops-id' },
        },
      },
    }

    expect(ensureDingtalkChannelConfigNode(channels)).toEqual({
      clientId: 'new-id',
      clientSecret: 'legacy-secret',
      accounts: {
        ops: { clientId: 'ops-id' },
        main: { clientId: 'main-id' },
      },
    })
  })
})
