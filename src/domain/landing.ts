import type { AppState } from './appState'
import type { NavPage } from './navigation'

export function resolveLandingPage(state: AppState): NavPage {
  if (state === 'NO_TARGET' || state === 'NEED_INSTALL') return 'overview'
  if (state === 'NEED_CONFIG') return 'ai-config'
  if (state === 'ERROR') return 'diagnostics'
  return 'overview'
}
