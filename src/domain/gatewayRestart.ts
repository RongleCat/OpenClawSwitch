export type GatewayRestartOs = 'windows' | 'macos' | 'linux'

export const GATEWAY_RESTART_DEBOUNCE_MS = 800

export const resolveGatewayRestartCommand = (os: GatewayRestartOs): string =>
  os === 'windows' ? 'cmd /c openclaw gateway restart' : 'sh -c "openclaw gateway restart"'

export interface DebouncedGatewayRestartController {
  schedule: (task: () => Promise<void> | void) => void
  dispose: () => void
}

export const createDebouncedGatewayRestartController = (
  delayMs = GATEWAY_RESTART_DEBOUNCE_MS
): DebouncedGatewayRestartController => {
  let timer: ReturnType<typeof setTimeout> | null = null

  return {
    schedule(task) {
      if (timer) clearTimeout(timer)
      timer = setTimeout(() => {
        timer = null
        void task()
      }, delayMs)
    },
    dispose() {
      if (!timer) return
      clearTimeout(timer)
      timer = null
    },
  }
}
