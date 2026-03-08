export type GateState = 'NO_TARGET' | 'NEED_INSTALL' | 'NEED_CONFIG' | null

export const shouldShowDashboardButton = (gateState: GateState) =>
  gateState !== 'NEED_INSTALL'

