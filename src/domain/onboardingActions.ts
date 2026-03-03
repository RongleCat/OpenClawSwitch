export type OnboardingMode = 'local' | 'ssh'

export type OnboardingPrimaryAction =
  | 'connect_target'
  | 'run_full_install'
  | 'show_remote_guide'
  | 'go_config'

export interface OnboardingActionInput {
  mode: OnboardingMode
  openclawInstalled: boolean
  envConnected: boolean
}

export function getOnboardingPrimaryAction(
  input: OnboardingActionInput
): OnboardingPrimaryAction {
  if (!input.envConnected) return 'connect_target'
  if (input.mode === 'ssh') {
    return input.openclawInstalled ? 'go_config' : 'show_remote_guide'
  }
  return input.openclawInstalled ? 'go_config' : 'run_full_install'
}
