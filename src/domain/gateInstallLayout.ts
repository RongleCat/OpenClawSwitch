export type GateState = 'NO_TARGET' | 'NEED_INSTALL' | 'NEED_CONFIG' | null
export type EnvMode = 'local' | 'ssh'
export type GateNavPage = 'overview' | 'ai-config' | 'diagnostics' | 'channels' | 'settings'

export const resolveGateTopbarTitle = (gateState: GateState, targetMode: EnvMode | null) => {
  if (gateState === 'NEED_INSTALL' && targetMode === 'local') {
    return '准备 OpenClaw 运行环境 · 检查内置运行时并完成本地配置'
  }
  return '安装与接入'
}

export const shouldUseFixedGateInstallLayout = (gateState: GateState, targetMode: EnvMode | null) =>
  (gateState === 'NEED_INSTALL' || gateState === 'NEED_CONFIG') && targetMode === 'local'

export const shouldUseFixedMainContentLayout = (
  isGateActive: boolean,
  gateState: GateState,
  targetMode: EnvMode | null,
  activeNav: GateNavPage,
  quickSetupForcedOpen: boolean
) =>
  shouldUseFixedGateInstallLayout(gateState, targetMode)
  || quickSetupForcedOpen
  || (!isGateActive && (activeNav === 'channels' || activeNav === 'overview' || activeNav === 'diagnostics' || activeNav === 'ai-config'))

export const shouldRenderSidebar = (isGateActive: boolean, quickSetupForcedOpen: boolean) =>
  !isGateActive && !quickSetupForcedOpen

export const shouldRenderQuickSetupGuide = (
  isGateActive: boolean,
  gateState: GateState,
  targetMode: EnvMode | null,
  quickSetupForcedOpen: boolean,
  envReady: boolean
) =>
  envReady && (
    (isGateActive && gateState === 'NEED_CONFIG' && targetMode === 'local')
    || quickSetupForcedOpen
  )
