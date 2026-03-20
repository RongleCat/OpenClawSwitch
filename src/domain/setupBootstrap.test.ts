import { describe, expect, it } from 'vitest'
import { resolveSetupCompletion, resolveSetupRedirect } from './setupBootstrap'

describe('resolveSetupCompletion', () => {
  it('keeps explicit setup completion from preferences', () => {
    expect(resolveSetupCompletion(true, false)).toBe(true)
  })

  it('auto-completes setup when runtime detects an existing local environment', () => {
    expect(resolveSetupCompletion(false, true)).toBe(true)
    expect(resolveSetupCompletion(false, false)).toBe(false)
  })
})

describe('resolveSetupRedirect', () => {
  it('redirects incomplete sessions into setup', () => {
    expect(resolveSetupRedirect('/', false)).toBe('/setup')
    expect(resolveSetupRedirect('/models', false)).toBe('/setup')
  })

  it('redirects completed sessions away from setup', () => {
    expect(resolveSetupRedirect('/setup', true)).toBe('/')
  })

  it('keeps users on their current route when no redirect is needed', () => {
    expect(resolveSetupRedirect('/', true)).toBeNull()
    expect(resolveSetupRedirect('/setup', false)).toBeNull()
  })
})
