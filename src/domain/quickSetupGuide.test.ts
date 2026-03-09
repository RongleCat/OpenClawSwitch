import { describe, expect, it } from 'vitest'
import {
  QUICK_SETUP_PRIMARY_PROVIDER_IDS,
  QUICK_SETUP_STEPS,
  QUICK_SETUP_CHANNEL_PRESETS,
  QUICK_SETUP_PROVIDER_PRESETS,
  applyQuickSetupGatewayOptions,
  applyQuickSetupModelPreset,
  clearQuickSetupManagedChannels,
  canSkipQuickSetupStep,
  createQuickSetupCustomProviderPreset,
  findProviderPreset,
  getGatewayInstallPlan,
  sanitizeQuickSetupChannelConfig,
} from './quickSetupGuide'

describe('quick setup steps', () => {
  it('keeps three ordered guide steps', () => {
    expect(QUICK_SETUP_STEPS.map((step) => step.id)).toEqual(['model', 'channel', 'gateway'])
  })

  it('only allows skipping first two steps', () => {
    expect(canSkipQuickSetupStep('model')).toBe(true)
    expect(canSkipQuickSetupStep('channel')).toBe(true)
    expect(canSkipQuickSetupStep('gateway')).toBe(false)
  })
})

describe('provider presets', () => {
  it('includes China-friendly coding presets and deepseek', () => {
    expect(findProviderPreset('dashscope-coding')?.baseUrl).toBe('https://coding.dashscope.aliyuncs.com/v1')
    expect(findProviderPreset('tencent-coding')?.baseUrl).toBe('https://api.lkeap.cloud.tencent.com/coding/v3')
    expect(findProviderPreset('deepseek')?.baseUrl).toBe('https://api.deepseek.com/v1')
  })

  it('keeps the model step focused on primary presets first', () => {
    expect(QUICK_SETUP_PRIMARY_PROVIDER_IDS).toEqual([
      'dashscope-coding',
      'tencent-coding',
      'deepseek',
    ])
  })

  it('writes documented aliyun coding provider config and model allowlist', () => {
    const preset = findProviderPreset('dashscope-coding')
    expect(preset).toBeTruthy()

    const next = applyQuickSetupModelPreset(
      {
        models: {
          providers: {
            legacy: {
              baseUrl: 'https://legacy.example.com/v1',
            },
          },
        },
        agents: {
          defaults: {
            models: {
              'legacy/model-a': {},
            },
          },
        },
      },
      preset!,
      'test-key',
      'qwen3.5-plus'
    )

    expect(next.models?.mode).toBe('merge')
    expect(next.models?.providers?.bailian?.baseUrl).toBe('https://coding.dashscope.aliyuncs.com/v1')
    expect(next.models?.providers?.bailian?.apiKey).toBe('test-key')
    expect(next.models?.providers?.bailian?.api).toBe('openai-completions')
    expect(next.models?.providers?.bailian?.models?.map(model => model.id)).toContain('qwen3-coder-plus')
    expect(next.agents?.defaults?.model?.primary).toBe('bailian/qwen3.5-plus')
    expect(next.agents?.defaults?.models).toMatchObject({
      'legacy/model-a': {},
      'bailian/qwen3.5-plus': {},
      'bailian/qwen3-max-2026-01-23': {},
      'bailian/qwen3-coder-next': {},
      'bailian/qwen3-coder-plus': {},
      'bailian/MiniMax-M2.5': {},
      'bailian/glm-5': {},
      'bailian/glm-4.7': {},
      'bailian/kimi-k2.5': {},
    })
  })

  it('adds selected model into provider and allowlist when fetched model is outside preset catalog', () => {
    const preset = findProviderPreset('tencent-coding')
    expect(preset).toBeTruthy()

    const next = applyQuickSetupModelPreset({}, preset!, 'test-key', 'custom-coder')

    expect(next.models?.providers?.lkeap?.models?.map(model => model.id)).toContain('custom-coder')
    expect(next.agents?.defaults?.model?.primary).toBe('lkeap/custom-coder')
    expect(next.agents?.defaults?.models).toMatchObject({
      'lkeap/glm-5': {},
      'lkeap/custom-coder': {},
    })
  })

  it('includes the documented Tencent Coding model catalog', () => {
    const preset = findProviderPreset('tencent-coding')
    expect(preset).toBeTruthy()

    expect(preset?.suggestedModels.map((model) => model.id)).toEqual([
      'hunyuan-2.0-instruct',
      'hunyuan-2.0-thinking',
      'hunyuan-t1',
      'hunyuan-turbos',
      'minimax-m2.5',
      'kimi-k2.5',
      'glm-5',
    ])

    expect(preset?.providerModels.map((model) => model.id)).toEqual([
      'hunyuan-2.0-instruct',
      'hunyuan-2.0-thinking',
      'hunyuan-t1',
      'hunyuan-turbos',
      'minimax-m2.5',
      'kimi-k2.5',
      'glm-5',
    ])
  })

  it('keeps preset providers ahead of the custom provider entry', () => {
    expect(QUICK_SETUP_PROVIDER_PRESETS.map((preset) => preset.id)).toEqual([
      'dashscope-coding',
      'tencent-coding',
      'deepseek',
      'dashscope',
      'hunyuan',
      'custom',
    ])
  })

  it('builds a custom provider preset for the simplified quick setup flow', () => {
    const preset = createQuickSetupCustomProviderPreset({
      providerName: 'my-openai',
      baseUrl: 'https://example.com/v1',
      selectedModelId: 'custom-main',
    })

    const next = applyQuickSetupModelPreset({}, preset, 'sk-custom', 'custom-main')

    expect(preset.id).toBe('custom')
    expect(preset.name).toBe('my-openai')
    expect(preset.baseUrl).toBe('https://example.com/v1')
    expect(preset.suggestedModels).toEqual([{ id: 'custom-main', name: 'custom-main' }])
    expect(next.models?.providers?.['my-openai']).toEqual({
      baseUrl: 'https://example.com/v1',
      apiKey: 'sk-custom',
      api: 'openai-completions',
      models: [{ id: 'custom-main', name: 'custom-main' }],
    })
    expect(next.agents?.defaults?.model?.primary).toBe('my-openai/custom-main')
  })
})

describe('channel presets', () => {
  it('keeps quick setup channels in the requested order', () => {
    expect(QUICK_SETUP_CHANNEL_PRESETS.map((preset) => preset.id)).toEqual([
      'feishu',
      'wecom',
      'qq',
      'dingtalk',
    ])
  })

  it('keeps qq quick setup on separate app id and app secret fields', () => {
    const preset = QUICK_SETUP_CHANNEL_PRESETS.find((item) => item.id === 'qq')

    expect(preset).toMatchObject({
      id: 'qq',
      placeholderLabel: 'App ID',
      secretLabel: 'App Secret',
    })
  })
})

describe('clearQuickSetupManagedChannels', () => {
  it('removes only quick-setup managed channels and preserves unrelated channels', () => {
    const next = clearQuickSetupManagedChannels({
      channels: {
        wecom: { botId: 'bot-id' },
        qqbot: { appId: 'qq-app-id' },
        dingtalk: { clientId: 'ding-id' },
        feishu: { appId: 'cli_xxx' },
        whatsapp: { sessionDir: '/tmp/wa' },
      },
    })

    expect(next).toEqual({
      channels: {
        whatsapp: { sessionDir: '/tmp/wa' },
      },
    })
  })
})

describe('sanitizeQuickSetupChannelConfig', () => {
  it('removes invalid default channel stubs before gateway startup', () => {
    const next = sanitizeQuickSetupChannelConfig({
      channels: {
        telegram: { enabled: false },
        discord: { enabled: false, dm: { policy: 'pairing' } },
        slack: { enabled: false, mode: 'http', webhookPath: '/webhooks/slack' },
        feishu: { enabled: false, connectionMode: 'websocket' },
        wecom: { enabled: false, dmPolicy: 'pairing' },
        qqbot: { enabled: false, dmPolicy: 'pairing' },
        dingtalk: { enabled: false, messageType: 'markdown' },
        whatsapp: { sessionDir: '/tmp/wa' },
      },
    })

    expect(next).toEqual({
      channels: {
        telegram: { enabled: false },
        discord: { enabled: false, dm: { policy: 'pairing' } },
        slack: { enabled: false, mode: 'http', webhookPath: '/webhooks/slack' },
        whatsapp: { sessionDir: '/tmp/wa' },
      },
    })
  })

  it('preserves fully configured channels', () => {
    const next = sanitizeQuickSetupChannelConfig({
      channels: {
        slack: {
          enabled: true,
          mode: 'http',
          signingSecret: 'secret',
          botToken: 'xoxb-token',
        },
        feishu: {
          enabled: true,
          appId: 'app-id',
          appSecret: 'app-secret',
        },
        wecom: {
          enabled: true,
          botId: 'bot-id',
          secret: 'bot-secret',
        },
        qqbot: {
          enabled: true,
          token: '1903108956:test-qq-token',
        },
        'dingtalk-connector': {
          enabled: true,
          clientId: 'client-id',
          clientSecret: 'client-secret',
        },
      },
    })

    expect(next).toEqual({
      channels: {
        slack: {
          enabled: true,
          mode: 'http',
          signingSecret: 'secret',
          botToken: 'xoxb-token',
        },
        feishu: {
          enabled: true,
          appId: 'app-id',
          appSecret: 'app-secret',
        },
        wecom: {
          enabled: true,
          botId: 'bot-id',
          secret: 'bot-secret',
        },
        qqbot: {
          enabled: true,
          token: '1903108956:test-qq-token',
        },
        'dingtalk-connector': {
          enabled: true,
          clientId: 'client-id',
          clientSecret: 'client-secret',
        },
      },
    })
  })

  it('keeps legacy qq credentials when appId and clientSecret are both present', () => {
    const next = sanitizeQuickSetupChannelConfig({
      channels: {
        qqbot: {
          enabled: true,
          appId: '1903108956',
          clientSecret: 'legacy-secret',
        },
      },
    })

    expect(next).toEqual({
      channels: {
        qqbot: {
          enabled: true,
          appId: '1903108956',
          clientSecret: 'legacy-secret',
        },
      },
    })
  })
})

describe('gateway install plan', () => {
  it('returns nssm-based wording on windows', () => {
    const plan = getGatewayInstallPlan('windows')
    expect(plan.title).toContain('Windows')
    expect(plan.summary).toContain('NSSM')
    expect(plan.commands).toEqual(['内置 NSSM 安装服务', '启动 OpenClaw Gateway 服务'])
  })

  it('returns shell commands on unix platforms', () => {
    expect(getGatewayInstallPlan('macos').commands).toEqual(['openclaw gateway install', 'openclaw gateway start'])
    expect(getGatewayInstallPlan('linux').commands).toEqual(['openclaw gateway install', 'openclaw gateway start'])
  })
})

describe('quick setup gateway options', () => {
  it('writes browser, full tools profile, and internal hooks defaults', () => {
    const next = applyQuickSetupGatewayOptions(
      {
        gateway: { mode: 'local' },
      },
      {
        browserDefaultProfileEnabled: true,
        toolsFullProfileEnabled: true,
      }
    )

    expect(next.browser).toEqual({ defaultProfile: 'openclaw' })
    expect(next.tools).toEqual({ profile: 'full' })
    expect(next.hooks).toEqual({
      internal: {
        enabled: true,
        entries: {
          'boot-md': { enabled: true },
          'bootstrap-extra-files': { enabled: true },
          'command-logger': { enabled: true },
          'session-memory': { enabled: true },
        },
      },
    })
  })

  it('removes managed browser and tools keys when toggles are disabled, while preserving unrelated fields and hooks', () => {
    const next = applyQuickSetupGatewayOptions(
      {
        browser: {
          defaultProfile: 'openclaw',
          keepCookies: true,
        },
        tools: {
          profile: 'full',
          extra: 'keep-me',
        },
        hooks: {
          external: {
            enabled: false,
          },
          internal: {
            entries: {
              custom: { enabled: false },
            },
          },
        },
      },
      {
        browserDefaultProfileEnabled: false,
        toolsFullProfileEnabled: false,
      }
    )

    expect(next.browser).toEqual({ keepCookies: true })
    expect(next.tools).toEqual({ extra: 'keep-me' })
    expect(next.hooks).toEqual({
      external: {
        enabled: false,
      },
      internal: {
        enabled: true,
        entries: {
          custom: { enabled: false },
          'boot-md': { enabled: true },
          'bootstrap-extra-files': { enabled: true },
          'command-logger': { enabled: true },
          'session-memory': { enabled: true },
        },
      },
    })
  })
})
