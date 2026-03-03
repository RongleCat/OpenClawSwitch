export type AppState =
  | 'NO_TARGET'
  | 'NEED_INSTALL'
  | 'NEED_CONFIG'
  | 'READY'
  | 'DEGRADED'
  | 'ERROR'

export interface AppStateInput {
  envConnected: boolean
  openclawInstalled: boolean
  configLoaded: boolean
  primaryModelValid: boolean
  gatewayReachable: boolean
  lastActionFailed: boolean
}

export function deriveAppState(input: AppStateInput): AppState {
  if (!input.envConnected) return 'NO_TARGET'
  if (!input.openclawInstalled) return 'NEED_INSTALL'
  // 门禁只要求配置文件存在；主模型有效性放到后续配置页与诊断页提示
  if (!input.configLoaded) return 'NEED_CONFIG'
  if (!input.gatewayReachable) return 'ERROR'
  if (input.lastActionFailed) return 'DEGRADED'
  return 'READY'
}
