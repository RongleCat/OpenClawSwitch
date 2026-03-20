import { beforeEach, describe, expect, it, vi } from 'vitest'
import { getGatewayStatus, resolveGatewayStatusSources, type GatewayStatus, type SystemGatewayStatus } from './runtime'

const { invokeSafe } = vi.hoisted(() => ({
  invokeSafe: vi.fn(),
}))

vi.mock('@/lib/desktop', () => ({
  invokeSafe,
}))

const stoppedManagedStatus: GatewayStatus = {
  state: 'stopped',
  pid: null,
  url: 'http://127.0.0.1:18789',
  message: null,
}

describe('resolveGatewayStatusSources', () => {
  it('keeps a managed running gateway as running', () => {
    expect(
      resolveGatewayStatusSources({
        managed: {
          state: 'running',
          pid: 123,
          url: 'http://127.0.0.1:18789',
          message: 'managed gateway',
        },
        reachable: false,
        system: null,
      })
    ).toEqual({
      state: 'running',
      pid: 123,
      url: 'http://127.0.0.1:18789',
      message: 'managed gateway',
    })
  })

  it('reports running when the system openclaw service is running even without a managed child', () => {
    const systemStatus: SystemGatewayStatus = {
      available: true,
      state: 'running',
      pid: 12527,
      url: 'ws://127.0.0.1:18789',
      message: 'LaunchAgent is running',
    }

    expect(
      resolveGatewayStatusSources({
        managed: stoppedManagedStatus,
        reachable: true,
        system: systemStatus,
      })
    ).toEqual({
      state: 'running',
      pid: 12527,
      url: 'http://127.0.0.1:18789',
      message: 'LaunchAgent is running',
    })
  })

  it('falls back to a network-based running status when the local gateway port is reachable', () => {
    expect(
      resolveGatewayStatusSources({
        managed: stoppedManagedStatus,
        reachable: true,
        system: null,
      })
    ).toEqual({
      state: 'running',
      pid: null,
      url: 'http://127.0.0.1:18789',
      message: 'Detected a reachable OpenClaw gateway on the default local address.',
    })
  })

  it('keeps the gateway stopped when no signal indicates a running service', () => {
    expect(
      resolveGatewayStatusSources({
        managed: stoppedManagedStatus,
        reachable: false,
        system: {
          available: true,
          state: 'stopped',
          pid: null,
          url: 'ws://127.0.0.1:18789',
          message: 'LaunchAgent is not running',
        },
      })
    ).toEqual(stoppedManagedStatus)
  })
})

describe('getGatewayStatus', () => {
  beforeEach(() => {
    invokeSafe.mockReset()
  })

  it('skips the system gateway probe during a lightweight refresh', async () => {
    invokeSafe.mockImplementation(async (command: string, _args: unknown, fallback: unknown) => {
      if (command === 'get_gateway_status') {
        return stoppedManagedStatus
      }

      if (command === 'health_check_gateway') {
        return true
      }

      if (command === 'get_system_gateway_status') {
        return {
          available: true,
          state: 'running',
          pid: 42,
          url: 'ws://127.0.0.1:18789',
          message: 'system gateway',
        } satisfies SystemGatewayStatus
      }

      return fallback
    })

    const lightweightGetGatewayStatus = getGatewayStatus as (
      options: { includeSystemStatus: boolean }
    ) => Promise<GatewayStatus>
    const status = await lightweightGetGatewayStatus({ includeSystemStatus: false })

    expect(status).toEqual({
      state: 'running',
      pid: null,
      url: 'http://127.0.0.1:18789',
      message: 'Detected a reachable OpenClaw gateway on the default local address.',
    })
    expect(invokeSafe).toHaveBeenCalledTimes(2)
    expect(invokeSafe).not.toHaveBeenCalledWith('get_system_gateway_status', undefined, null)
  })
})
