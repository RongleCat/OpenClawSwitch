import { describe, expect, it } from 'vitest'
import { shouldKeepAutoScroll } from './terminalAutoScroll'

describe('shouldKeepAutoScroll', () => {
  it('keeps auto scroll when user has not manually scrolled up', () => {
    expect(
      shouldKeepAutoScroll({
        scrollTop: 700,
        clientHeight: 300,
        scrollHeight: 1000,
        wasAutoScrollEnabled: true,
      })
    ).toBe(true)
  })

  it('stops auto scroll after user scrolls away from bottom', () => {
    expect(
      shouldKeepAutoScroll({
        scrollTop: 450,
        clientHeight: 300,
        scrollHeight: 1000,
        wasAutoScrollEnabled: true,
      })
    ).toBe(false)
  })

  it('resumes auto scroll when user scrolls back to bottom', () => {
    expect(
      shouldKeepAutoScroll({
        scrollTop: 700,
        clientHeight: 300,
        scrollHeight: 1000,
        wasAutoScrollEnabled: false,
      })
    ).toBe(true)
  })
})
