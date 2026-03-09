import { describe, expect, it } from 'vitest'
import {
  DEFAULT_GATEWAY_READY_OPTIONS,
  GATEWAY_READY_INTERVAL_MS,
  GATEWAY_READY_MAX_ATTEMPTS,
  GATEWAY_READY_TIMEOUT_MS,
  waitForGatewayReady,
} from './gatewayStartup'

describe('gateway startup defaults', () => {
  it('uses a 3 minute readiness window', () => {
    expect(GATEWAY_READY_TIMEOUT_MS).toBe(180000)
    expect(GATEWAY_READY_INTERVAL_MS).toBe(1000)
    expect(GATEWAY_READY_MAX_ATTEMPTS).toBe(180)
    expect(DEFAULT_GATEWAY_READY_OPTIONS).toEqual({
      maxAttempts: 180,
      intervalMs: 1000,
    })
  })
})

describe('waitForGatewayReady', () => {
  it('returns true once health check succeeds', async () => {
    let attempts = 0

    const ready = await waitForGatewayReady(
      async () => {
        attempts += 1
        return attempts >= 3
      },
      {
        maxAttempts: 5,
        intervalMs: 0,
        sleep: async () => {},
      }
    )

    expect(ready).toBe(true)
    expect(attempts).toBe(3)
  })

  it('returns false after exhausting all attempts', async () => {
    let attempts = 0

    const ready = await waitForGatewayReady(
      async () => {
        attempts += 1
        return false
      },
      {
        maxAttempts: 4,
        intervalMs: 0,
        sleep: async () => {},
      }
    )

    expect(ready).toBe(false)
    expect(attempts).toBe(4)
  })
})
