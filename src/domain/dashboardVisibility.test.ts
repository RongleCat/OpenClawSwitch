import { describe, expect, it } from 'vitest'
import { shouldShowDashboardButton, type GateState } from './dashboardVisibility'

describe('shouldShowDashboardButton', () => {
  it('hides dashboard in install step', () => {
    expect(shouldShowDashboardButton('NEED_INSTALL')).toBe(false)
  })

  it('shows dashboard for other gate states', () => {
    const states: Array<Exclude<GateState, 'NEED_INSTALL'>> = ['NO_TARGET', 'NEED_CONFIG', null]
    for (const state of states) {
      expect(shouldShowDashboardButton(state)).toBe(true)
    }
  })
})
