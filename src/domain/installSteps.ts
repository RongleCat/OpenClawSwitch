export type InstallStepStatus = 'pending' | 'running' | 'success' | 'error'

export interface InstallStepItem {
  name: string
  status: InstallStepStatus
  duration: number
}

const createStep = (name: string): InstallStepItem => ({
  name,
  status: 'pending',
  duration: 0,
})

export const buildInstallSteps = (_isWindows: boolean): InstallStepItem[] => [
  createStep('环境检测'),
  createStep('安装 Git'),
  createStep('安装 Node.js'),
  createStep('准备 OpenClaw 运行环境'),
  createStep('验证安装'),
]

export const buildInstallStepIndexMap = (_isWindows: boolean): Record<string, number> => ({
  check: 0,
  install_git: 1,
  install_node: 2,
  install_openclaw: 3,
  verify: 4,
})
