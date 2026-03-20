type DeferredFrameScheduler = (callback: FrameRequestCallback) => number
type DeferredFrameCanceler = (handle: number) => void
type DeferredTimeoutScheduler = (callback: () => void, delayMs: number) => number
type DeferredTimeoutCanceler = (handle: number) => void

export interface DeferredTaskEnvironment {
  requestAnimationFrame?: DeferredFrameScheduler
  cancelAnimationFrame?: DeferredFrameCanceler
  setTimeout?: DeferredTimeoutScheduler
  clearTimeout?: DeferredTimeoutCanceler
}

export const scheduleDeferredTask = (
  task: () => void,
  environment: DeferredTaskEnvironment = {}
) => {
  const requestFrame = environment.requestAnimationFrame ?? window.requestAnimationFrame.bind(window)
  const cancelFrame = environment.cancelAnimationFrame ?? window.cancelAnimationFrame.bind(window)
  const scheduleTimeout = environment.setTimeout ?? window.setTimeout.bind(window)
  const cancelTimeout = environment.clearTimeout ?? window.clearTimeout.bind(window)

  let timeoutId: number | null = null
  let frameId: number | null = requestFrame(() => {
    frameId = null
    timeoutId = scheduleTimeout(() => {
      timeoutId = null
      task()
    }, 0)
  })

  return () => {
    if (frameId !== null) {
      cancelFrame(frameId)
      frameId = null
    }

    if (timeoutId !== null) {
      cancelTimeout(timeoutId)
      timeoutId = null
    }
  }
}
