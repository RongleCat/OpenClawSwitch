import { describe, expect, it } from 'vitest'
import { waitForGatewayReady } from './gatewayStartup'

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
