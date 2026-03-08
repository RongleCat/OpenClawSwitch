import { describe, expect, it } from 'vitest'
import {
  QUICK_SETUP_STEPS,
  canSkipQuickSetupStep,
  findProviderPreset,
  getGatewayInstallPlan,
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
