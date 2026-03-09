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
