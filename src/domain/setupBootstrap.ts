export const resolveSetupCompletion = (
  storedSetupComplete: boolean,
  runtimeShouldSkipSetup: boolean
): boolean => storedSetupComplete || runtimeShouldSkipSetup

export const resolveSetupRedirect = (
  pathname: string,
  setupComplete: boolean
): string | null => {
  if (!setupComplete && pathname !== '/setup') {
    return '/setup'
  }

  if (setupComplete && pathname === '/setup') {
    return '/'
  }

  return null
}
