const defaultSleep = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms))

export const GATEWAY_READY_TIMEOUT_MS = 3 * 60 * 1000
export const GATEWAY_READY_INTERVAL_MS = 1000
export const GATEWAY_READY_MAX_ATTEMPTS = GATEWAY_READY_TIMEOUT_MS / GATEWAY_READY_INTERVAL_MS
export const DEFAULT_GATEWAY_READY_OPTIONS = {
  maxAttempts: GATEWAY_READY_MAX_ATTEMPTS,
  intervalMs: GATEWAY_READY_INTERVAL_MS,
} as const

interface WaitForGatewayReadyOptions {
  maxAttempts?: number
  intervalMs?: number
  sleep?: (ms: number) => Promise<void>
}

export const waitForGatewayReady = async (
  checkHealth: () => Promise<boolean>,
  options: WaitForGatewayReadyOptions = {}
) => {
  const {
    maxAttempts = DEFAULT_GATEWAY_READY_OPTIONS.maxAttempts,
    intervalMs = DEFAULT_GATEWAY_READY_OPTIONS.intervalMs,
    sleep = defaultSleep,
  } = options

  for (let attempt = 1; attempt <= maxAttempts; attempt += 1) {
    if (await checkHealth()) {
      return true
    }

    if (attempt < maxAttempts) {
      await sleep(intervalMs)
    }
  }

  return false
}
