export type ValidateMode = 'local' | 'ssh'

export function nextValidateSteps(mode: ValidateMode): string[] {
  if (mode === 'ssh') {
    return ['save_remote', 'remote_restart', 'remote_health_check']
  }

  return ['save', 'restart', 'health_check']
}
