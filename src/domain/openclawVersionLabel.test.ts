import { describe, expect, it } from 'vitest'
import { formatOpenClawVersionLabel } from './openclawVersionLabel'

describe('formatOpenClawVersionLabel', () => {
  it('extracts the date portion from the bundled OpenClaw version string', () => {
    expect(formatOpenClawVersionLabel('OpenClaw 2026.3.8 (3caab92)')).toBe('2026.3.8')
  })

  it('keeps plain date versions as-is', () => {
    expect(formatOpenClawVersionLabel('2026.03.10')).toBe('2026.03.10')
  })

  it('falls back to the original value when no date is present', () => {
    expect(formatOpenClawVersionLabel('nightly-build')).toBe('nightly-build')
  })

  it('shows placeholder when version is empty', () => {
    expect(formatOpenClawVersionLabel('')).toBe('--')
    expect(formatOpenClawVersionLabel(null)).toBe('--')
  })
})
