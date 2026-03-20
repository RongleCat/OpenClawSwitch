export interface GatewayServiceActionInput {
  isWindows: boolean
  envMode: 'local' | 'ssh'
  gatewayServiceInstalled: boolean
  gatewayReachable?: boolean
}

export interface GatewayQuickActionStateInput {
  actionId: string
  baseDisabled: boolean
  pendingActionId: string | null
}

export const shouldShowInstallGatewayServiceAction = ({
  isWindows,
  envMode,
  gatewayServiceInstalled,
  gatewayReachable = false,
}: GatewayServiceActionInput) =>
  isWindows && envMode === 'local' && !gatewayServiceInstalled && !gatewayReachable

export const resolveGatewayQuickActionGridColumns = (actionCount: number) =>
  Math.min(Math.max(actionCount, 1), 4)

export const resolveGatewayQuickActionState = ({
  actionId,
  baseDisabled,
  pendingActionId,
}: GatewayQuickActionStateInput) => ({
  loading: pendingActionId === actionId,
  disabled: baseDisabled || pendingActionId !== null,
})

export const resolveGatewayQuickActionLabel = (
  actionId: string,
  label: string,
  loading: boolean
) => {
  if (!loading) {
    return label
  }

  const loadingLabels: Record<string, string> = {
    start: '启动中...',
    restart: '重启中...',
    stop: '停止中...',
  }

  return loadingLabels[actionId] || `${label}中...`
}
