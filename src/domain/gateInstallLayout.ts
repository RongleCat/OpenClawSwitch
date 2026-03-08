export type GateState = 'NO_TARGET' | 'NEED_INSTALL' | 'NEED_CONFIG' | null
export type EnvMode = 'local' | 'ssh'

export const resolveGateTopbarTitle = (gateState: GateState, targetMode: EnvMode | null) => {
  if (gateState === 'NEED_INSTALL' && targetMode === 'local') {
    return '安装 OpenClaw · 自动检测环境并安装所有依赖'
  }
  return '安装与接入'
}

export const shouldUseFixedGateInstallLayout = (gateState: GateState, targetMode: EnvMode | null) =>
  gateState === 'NEED_INSTALL' && targetMode === 'local'

