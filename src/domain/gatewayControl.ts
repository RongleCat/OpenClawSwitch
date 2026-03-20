export type GatewayControlOs = 'windows' | 'macos' | 'linux'
export type GatewayControlMode = 'windows-external-cli' | 'bundled-runtime'
export interface GatewayControlCommands {
  start: 'start_external_gateway' | 'start_gateway'
  restart: 'restart_external_gateway' | 'restart_gateway'
  stop: 'stop_external_gateway' | 'stop_gateway'
}

interface ResolveGatewayControlModeInput {
  os: GatewayControlOs
  localOpenclawAvailable: boolean
}

export const resolveGatewayControlMode = ({
  os,
  localOpenclawAvailable,
}: ResolveGatewayControlModeInput): GatewayControlMode =>
  os === 'windows' && localOpenclawAvailable ? 'windows-external-cli' : 'bundled-runtime'

export const resolveGatewayControlCommands = (
  mode: GatewayControlMode
): GatewayControlCommands =>
  mode === 'windows-external-cli'
    ? {
        start: 'start_external_gateway',
        restart: 'restart_external_gateway',
        stop: 'stop_external_gateway',
      }
    : {
        start: 'start_gateway',
        restart: 'restart_gateway',
        stop: 'stop_gateway',
      }
