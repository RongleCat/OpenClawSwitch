import { afterEach, describe, expect, it, vi } from 'vitest'
import {
  GATEWAY_RESTART_DEBOUNCE_MS,
  createDebouncedGatewayRestartController,
  resolveGatewayRestartCommand,
} from './gatewayRestart'

describe('resolveGatewayRestartCommand', () => {
  it('returns the documented local restart command for each desktop platform', () => {
    expect(resolveGatewayRestartCommand('windows')).toBe('cmd /c openclaw gateway restart')
    expect(resolveGatewayRestartCommand('macos')).toBe('sh -c "openclaw gateway restart"')
    expect(resolveGatewayRestartCommand('linux')).toBe('sh -c "openclaw gateway restart"')
  })
})

describe('createDebouncedGatewayRestartController', () => {
  afterEach(() => {
    vi.useRealTimers()
  })

  it('runs only the latest scheduled restart after the debounce window', async () => {
    vi.useFakeTimers()
    const calls: number[] = []
    const controller = createDebouncedGatewayRestartController(200)

    controller.schedule(async () => {
      calls.push(1)
    })
    controller.schedule(async () => {
      calls.push(2)
    })

    await vi.advanceTimersByTimeAsync(199)
    expect(calls).toEqual([])

    await vi.advanceTimersByTimeAsync(1)
    expect(calls).toEqual([2])
  })

  it('cancels a pending restart when disposed', async () => {
    vi.useFakeTimers()
    const calls: number[] = []
    const controller = createDebouncedGatewayRestartController(GATEWAY_RESTART_DEBOUNCE_MS)

    controller.schedule(async () => {
      calls.push(1)
    })
    controller.dispose()

    await vi.advanceTimersByTimeAsync(GATEWAY_RESTART_DEBOUNCE_MS)
    expect(calls).toEqual([])
  })
})
