import { describe, expect, it } from 'vitest'
import {
  MANAGED_MESSAGE_CHANNEL_IDS,
  buildMessageChannelAccountLabel,
  collectMessageChannelAccountIds,
  extractMessageChannelPublicConfig,
  isMessageChannelAccountIdValid,
  isMessageChannelConfigured,
  removeMessageChannelAccountConfig,
  saveMessageChannelAccountConfig,
  saveMessageChannelDefaultAccountConfig,
  saveMessageChannelPublicConfig,
  supportsMessageChannelMultipleAccounts,
} from './messageChannelAccounts'

describe('MANAGED_MESSAGE_CHANNEL_IDS', () => {
  it('keeps plugin-backed channels and classic channels in supported order', () => {
    expect(MANAGED_MESSAGE_CHANNEL_IDS).toEqual([
      'feishu',
      'wecom',
      'qq',
      'dingtalk',
      'telegram',
      'discord',
      'slack',
    ])
  })
})

describe('supportsMessageChannelMultipleAccounts', () => {
  it('disables multi-account mode for qq and wecom only', () => {
    expect(supportsMessageChannelMultipleAccounts('qq')).toBe(false)
    expect(supportsMessageChannelMultipleAccounts('wecom')).toBe(false)
    expect(supportsMessageChannelMultipleAccounts('feishu')).toBe(true)
    expect(supportsMessageChannelMultipleAccounts('telegram')).toBe(true)
  })
})

describe('collectMessageChannelAccountIds', () => {
  it('always includes default and sorts named accounts after it', () => {
    expect(
      collectMessageChannelAccountIds({
        accounts: {
          ops: {},
          main: {},
        },
      })
    ).toEqual(['default', 'main', 'ops'])
  })
})

describe('buildMessageChannelAccountLabel', () => {
  it('uses id, name, and credentials to distinguish accounts', () => {
    expect(
      buildMessageChannelAccountLabel('dingtalk', 'main', {
        name: '主账号',
        clientId: 'dingabcdefgh12345678',
      })
    ).toContain('main')

    expect(
      buildMessageChannelAccountLabel('dingtalk', 'main', {
        name: '主账号',
        clientId: 'dingabcdefgh12345678',
      })
    ).toContain('主账号')

    expect(
      buildMessageChannelAccountLabel('telegram', 'ops', {
        botToken: '123456789:ABCDEF1234567890',
      })
    ).toContain('123456')
  })
})

describe('isMessageChannelAccountIdValid', () => {
  it('accepts stable ids and rejects whitespace ids', () => {
    expect(isMessageChannelAccountIdValid('main')).toBe(true)
    expect(isMessageChannelAccountIdValid('coding-plan')).toBe(true)
    expect(isMessageChannelAccountIdValid('ops.bot')).toBe(true)
    expect(isMessageChannelAccountIdValid('bad id')).toBe(false)
    expect(isMessageChannelAccountIdValid('')).toBe(false)
  })
})

describe('isMessageChannelConfigured', () => {
  it('ignores empty accounts and only marks configured when required credentials exist', () => {
    expect(
      isMessageChannelConfigured('feishu', {
        accounts: {
          empty: {},
          partial: { appId: 'cli_xxx' },
        },
      })
    ).toBe(false)

    expect(
      isMessageChannelConfigured('feishu', {
        accounts: {
          main: { appId: 'cli_xxx', appSecret: 'secret' },
        },
      })
    ).toBe(true)

    expect(
      isMessageChannelConfigured('slack', {
        accounts: {
          empty: {},
          main: { botToken: 'xoxb-token' },
        },
      })
    ).toBe(true)

    expect(
      isMessageChannelConfigured('wecom', {
        botId: 'wecom-bot-id',
        secret: 'wecom-secret',
      })
    ).toBe(true)

    expect(
      isMessageChannelConfigured('qq', {
        token: '1903108956:test-qq-token',
      })
    ).toBe(true)
  })
})

describe('saveMessageChannelAccountConfig', () => {
  it('writes named accounts under accounts and strips shared enabled flag', () => {
    const root: Record<string, unknown> = {
      channels: {
        telegram: {
          enabled: true,
        },
      },
    }

    saveMessageChannelAccountConfig(root, 'telegram', 'ops', {
      enabled: false,
      botToken: '123456789:ABCDEF1234567890',
      dmPolicy: 'pairing',
    })

    expect(root).toEqual({
      channels: {
        telegram: {
          enabled: true,
          accounts: {
            ops: {
              botToken: '123456789:ABCDEF1234567890',
            },
          },
        },
      },
    })
  })

  it('merges unknown dingtalk plugin fields while replacing editable fields', () => {
    const root: Record<string, unknown> = {
      channels: {
        dingtalk: {
          accounts: {
            ops: {
              clientId: 'old',
              showThinking: true,
              mediaMaxMb: 20,
            },
          },
        },
      },
    }

    saveMessageChannelAccountConfig(root, 'dingtalk', 'ops', {
      enabled: true,
      clientId: 'new',
      debug: true,
      dmPolicy: 'allowlist',
    })

    expect(root).toEqual({
      channels: {
        dingtalk: {
          accounts: {
            ops: {
              clientId: 'new',
              showThinking: true,
              mediaMaxMb: 20,
            },
          },
        },
      },
    })
  })
})

describe('saveMessageChannelDefaultAccountConfig', () => {
  it('replaces only default-account fields on the root channel node', () => {
    const root: Record<string, unknown> = {
      channels: {
        telegram: {
          enabled: true,
          botToken: 'legacy-root-token',
          dmPolicy: 'pairing',
          groupPolicy: 'allowlist',
          accounts: {
            ops: {
              botToken: 'ops-token',
            },
          },
        },
      },
    }

    saveMessageChannelDefaultAccountConfig(root, 'telegram', {
      enabled: false,
      botToken: 'new-default-token',
      dmPolicy: 'open',
    })

    expect(root).toEqual({
      channels: {
        telegram: {
          enabled: false,
          botToken: 'new-default-token',
          dmPolicy: 'pairing',
          groupPolicy: 'allowlist',
          accounts: {
            ops: {
              botToken: 'ops-token',
            },
          },
        },
      },
    })
  })

  it('writes QQ default credentials into channels.qqbot instead of channels.qq', () => {
    const root: Record<string, unknown> = { channels: {} }

    saveMessageChannelDefaultAccountConfig(root, 'qq', {
      enabled: true,
      token: '1903108956:test-qq-token',
      markdownSupport: false,
    })

    expect(root).toEqual({
      channels: {
        qqbot: {
          enabled: true,
          token: '1903108956:test-qq-token',
        },
      },
    })
  })
})

describe('extractMessageChannelPublicConfig', () => {
  it('keeps only public fields for dingtalk shared config', () => {
    expect(
      extractMessageChannelPublicConfig('dingtalk', {
        enabled: true,
        clientId: 'cid',
        clientSecret: 'secret',
        robotCode: 'robot',
        corpId: 'corp',
        agentId: 'main',
        dmPolicy: 'open',
        groupPolicy: 'allowlist',
        messageType: 'card',
        cardTemplateKey: 'content',
        showThinking: true,
        mediaMaxMb: 20,
        accounts: {
          ops: { clientId: 'ops' },
        },
      })
    ).toEqual({
      dmPolicy: 'open',
      groupPolicy: 'allowlist',
      messageType: 'card',
      cardTemplateKey: 'content',
      showThinking: true,
      mediaMaxMb: 20,
    })
  })
})

describe('saveMessageChannelPublicConfig', () => {
  it('replaces telegram shared fields without overwriting credentials', () => {
    const root: Record<string, unknown> = {
      channels: {
        telegram: {
          enabled: true,
          botToken: 'root-token',
          dmPolicy: 'pairing',
          defaultTo: 'old-default',
          accounts: {
            ops: {
              botToken: 'ops-token',
            },
          },
        },
      },
    }

    saveMessageChannelPublicConfig(root, 'telegram', {
      enabled: false,
      botToken: 'ignored-token',
      dmPolicy: 'open',
      groupPolicy: 'allowlist',
    })

    expect(root).toEqual({
      channels: {
        telegram: {
          enabled: true,
          botToken: 'root-token',
          dmPolicy: 'open',
          groupPolicy: 'allowlist',
          accounts: {
            ops: {
              botToken: 'ops-token',
            },
          },
        },
      },
    })
  })

  it('replaces discord shared branches so cleared nested values do not linger', () => {
    const root: Record<string, unknown> = {
      channels: {
        discord: {
          token: 'root-token',
          dm: {
            policy: 'allowlist',
            allowFrom: ['old-user'],
          },
          groupPolicy: 'allowlist',
          replyToMode: 'first',
          accounts: {
            ops: {
              token: 'ops-token',
            },
          },
        },
      },
    }

    saveMessageChannelPublicConfig(root, 'discord', {
      token: 'ignored-token',
      dm: {
        policy: 'open',
      },
      replyToMode: 'all',
    })

    expect(root).toEqual({
      channels: {
        discord: {
          token: 'root-token',
          dm: {
            policy: 'open',
          },
          replyToMode: 'all',
          accounts: {
            ops: {
              token: 'ops-token',
            },
          },
        },
      },
    })
  })
})

describe('removeMessageChannelAccountConfig', () => {
  it('removes named accounts and clears defaultAccount when needed', () => {
    const root: Record<string, unknown> = {
      channels: {
        slack: {
          defaultAccount: 'ops',
          accounts: {
            ops: { botToken: 'xoxb-token' },
            qa: { botToken: 'xoxb-other' },
          },
        },
      },
    }

    removeMessageChannelAccountConfig(root, 'slack', 'ops')

    expect(root).toEqual({
      channels: {
        slack: {
          accounts: {
            qa: { botToken: 'xoxb-other' },
          },
        },
      },
    })
  })
})
