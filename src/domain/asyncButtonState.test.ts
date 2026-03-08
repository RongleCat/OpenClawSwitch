import { describe, expect, it } from 'vitest'
import {
  resolveAsyncButtonLabel,
  resolveAsyncButtonState,
  runAsyncOnce,
} from './asyncButtonState'

describe('resolveAsyncButtonState', () => {
  it('marks loading buttons as disabled', () => {
    expect(
      resolveAsyncButtonState({
        loading: true,
        baseDisabled: false,
      })
    ).toEqual({
      loading: true,
      disabled: true,
    })
  })

  it('preserves base disabled state', () => {
    expect(
      resolveAsyncButtonState({
        loading: false,
        baseDisabled: true,
      })
    ).toEqual({
      loading: false,
      disabled: true,
    })
  })
})

describe('resolveAsyncButtonLabel', () => {
  it('returns loading label while busy', () => {
    expect(
      resolveAsyncButtonLabel({
        loading: true,
        label: '刷新',
        loadingLabel: '刷新中...',
      })
    ).toBe('刷新中...')
  })

  it('returns default label when idle', () => {
    expect(
      resolveAsyncButtonLabel({
        loading: false,
        label: 'Dashboard',
        loadingLabel: '打开中...',
      })
    ).toBe('Dashboard')
  })
})

describe('runAsyncOnce', () => {
  it('skips execution while already running', async () => {
    let running = true
    let called = false

    const result = await runAsyncOnce({
      isRunning: () => running,
      setRunning: (next) => {
        running = next
      },
      action: async () => {
        called = true
      },
    })

    expect(result).toBeUndefined()
    expect(called).toBe(false)
  })

  it('sets and clears running flag around action', async () => {
    let running = false
    const states: boolean[] = []

    const result = await runAsyncOnce({
      isRunning: () => running,
      setRunning: (next) => {
        running = next
        states.push(next)
      },
      action: async () => 'done',
    })

    expect(result).toBe('done')
    expect(states).toEqual([true, false])
    expect(running).toBe(false)
  })
})
