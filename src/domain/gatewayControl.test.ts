import { describe, expect, it } from 'vitest'
import { resolveGatewayControlCommands, resolveGatewayControlMode } from './gatewayControl'

describe('resolveGatewayControlMode', () => {
  it('prefers external cli control on windows when a local openclaw installation is available', () => {
    expect(
      resolveGatewayControlMode({
        os: 'windows',
        localOpenclawAvailable: true,
      })
    ).toBe('windows-external-cli')
  })

  it('falls back to bundled runtime control on windows when external openclaw is unavailable', () => {
    expect(
      resolveGatewayControlMode({
        os: 'windows',
        localOpenclawAvailable: false,
      })
    ).toBe('bundled-runtime')
  })

  it('keeps bundled runtime control on non-windows platforms', () => {
    expect(
      resolveGatewayControlMode({
        os: 'macos',
        localOpenclawAvailable: true,
      })
    ).toBe('bundled-runtime')
    expect(
      resolveGatewayControlMode({
        os: 'linux',
        localOpenclawAvailable: true,
      })
    ).toBe('bundled-runtime')
  })
})

describe('resolveGatewayControlCommands', () => {
  it('returns external tauri command names for windows external cli mode', () => {
    expect(resolveGatewayControlCommands('windows-external-cli')).toEqual({
      start: 'start_external_gateway',
      restart: 'restart_external_gateway',
      stop: 'stop_external_gateway',
    })
  })

  it('returns bundled tauri command names for bundled runtime mode', () => {
    expect(resolveGatewayControlCommands('bundled-runtime')).toEqual({
      start: 'start_gateway',
      restart: 'restart_gateway',
      stop: 'stop_gateway',
    })
  })
})
