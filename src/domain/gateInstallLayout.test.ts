import { describe, expect, it } from 'vitest'
import { resolveGateTopbarTitle, shouldUseFixedGateInstallLayout } from './gateInstallLayout'

describe('resolveGateTopbarTitle', () => {
  it('returns merged install title in local install step', () => {
    expect(resolveGateTopbarTitle('NEED_INSTALL', 'local')).toBe('安装 OpenClaw · 自动检测环境并安装所有依赖')
  })

  it('returns default gate title for other states', () => {
    expect(resolveGateTopbarTitle('NO_TARGET', null)).toBe('安装与接入')
    expect(resolveGateTopbarTitle('NEED_CONFIG', 'local')).toBe('安装与接入')
    expect(resolveGateTopbarTitle(null, 'local')).toBe('安装与接入')
  })
})

describe('shouldUseFixedGateInstallLayout', () => {
  it('uses fixed layout only in local install step', () => {
    expect(shouldUseFixedGateInstallLayout('NEED_INSTALL', 'local')).toBe(true)
    expect(shouldUseFixedGateInstallLayout('NEED_INSTALL', 'ssh')).toBe(false)
    expect(shouldUseFixedGateInstallLayout('NO_TARGET', 'local')).toBe(false)
    expect(shouldUseFixedGateInstallLayout('NEED_CONFIG', 'local')).toBe(false)
  })
})
