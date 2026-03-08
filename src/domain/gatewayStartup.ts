const defaultSleep = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms))

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
    maxAttempts = 30,
    intervalMs = 2000,
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
