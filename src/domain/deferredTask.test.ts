import { describe, expect, it, vi } from 'vitest'
import { scheduleDeferredTask } from './deferredTask'

describe('scheduleDeferredTask', () => {
  it('does not run the task synchronously', () => {
    const calls: string[] = []

    scheduleDeferredTask(
      () => {
        calls.push('task')
      },
      {
        requestAnimationFrame: () => 1,
        cancelAnimationFrame: () => undefined,
        setTimeout: () => 1,
        clearTimeout: () => undefined,
      }
    )

    expect(calls).toEqual([])
  })

  it('runs the task after the animation frame and timeout complete', () => {
    let frameCallback: FrameRequestCallback | null = null
    let timeoutCallback: (() => void) | null = null
    const calls: string[] = []

    scheduleDeferredTask(
      () => {
        calls.push('task')
      },
      {
        requestAnimationFrame: (callback) => {
          frameCallback = callback
          return 1
        },
        cancelAnimationFrame: () => undefined,
        setTimeout: (callback: () => void) => {
          timeoutCallback = callback
          return 2
        },
        clearTimeout: () => undefined,
      }
    )

    expect(frameCallback).not.toBeNull()
    frameCallback!(16.7)
    expect(calls).toEqual([])

    expect(timeoutCallback).not.toBeNull()
    timeoutCallback!()
    expect(calls).toEqual(['task'])
  })

  it('cancels the deferred task before it runs', () => {
    let frameCallback: FrameRequestCallback | null = null
    const cancelAnimationFrame = vi.fn()
    const clearTimeout = vi.fn()
    const calls: string[] = []

    const dispose = scheduleDeferredTask(
      () => {
        calls.push('task')
      },
      {
        requestAnimationFrame: (callback) => {
          frameCallback = callback
          return 7
        },
        cancelAnimationFrame,
        setTimeout: () => 9,
        clearTimeout,
      }
    )

    dispose()
    expect(frameCallback).not.toBeNull()
    frameCallback!(16.7)

    expect(calls).toEqual([])
    expect(cancelAnimationFrame).toHaveBeenCalledWith(7)
    expect(clearTimeout).not.toHaveBeenCalled()
  })
})
