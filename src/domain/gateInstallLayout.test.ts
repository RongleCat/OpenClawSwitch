import { describe, expect, it } from 'vitest'
import {
  resolveGateTopbarTitle,
  shouldRenderQuickSetupGuide,
  shouldRenderSidebar,
  shouldUseFixedGateInstallLayout,
  shouldUseFixedMainContentLayout,
} from './gateInstallLayout'

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
  it('uses fixed layout for local install and quick setup steps', () => {
    expect(shouldUseFixedGateInstallLayout('NEED_INSTALL', 'local')).toBe(true)
    expect(shouldUseFixedGateInstallLayout('NEED_CONFIG', 'local')).toBe(true)
    expect(shouldUseFixedGateInstallLayout('NEED_INSTALL', 'ssh')).toBe(false)
    expect(shouldUseFixedGateInstallLayout('NO_TARGET', 'local')).toBe(false)
    expect(shouldUseFixedGateInstallLayout('NEED_CONFIG', 'ssh')).toBe(false)
  })
})

describe('shouldUseFixedMainContentLayout', () => {
  it('keeps quick setup debug view in fixed-height layout', () => {
    expect(shouldUseFixedMainContentLayout(true, 'NEED_CONFIG', 'local', 'overview', false)).toBe(true)
    expect(shouldUseFixedMainContentLayout(false, null, null, 'settings', true)).toBe(true)
    expect(shouldUseFixedMainContentLayout(false, null, null, 'settings', false)).toBe(false)
  })
})

describe('shouldRenderQuickSetupGuide', () => {
  it('supports gate flow and manual debug entry', () => {
    expect(shouldRenderQuickSetupGuide(true, 'NEED_CONFIG', 'local', false, true)).toBe(true)
    expect(shouldRenderQuickSetupGuide(false, null, null, true, true)).toBe(true)
    expect(shouldRenderQuickSetupGuide(false, null, null, true, false)).toBe(false)
    expect(shouldRenderQuickSetupGuide(true, 'NEED_CONFIG', 'ssh', false, true)).toBe(false)
  })

  it('allows forcing quick setup open after admin relaunch resume', () => {
    expect(shouldRenderQuickSetupGuide(false, null, 'local', true, true)).toBe(true)
    expect(shouldRenderQuickSetupGuide(false, 'NO_TARGET', 'local', true, true)).toBe(true)
  })
})

describe('shouldRenderSidebar', () => {
  it('hides the app sidebar while quick setup resume is forced open', () => {
    expect(shouldRenderSidebar(false, false)).toBe(true)
    expect(shouldRenderSidebar(true, false)).toBe(false)
    expect(shouldRenderSidebar(false, true)).toBe(false)
  })
})
