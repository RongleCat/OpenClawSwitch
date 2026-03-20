export interface OpenClawUninstallActionVisibilityInput {
  envMode: 'local' | 'ssh'
}

export interface OpenClawUninstallActionStateInput {
  envMode: 'local' | 'ssh'
  openclawInstalled: boolean
  loading: boolean
}

export interface OpenClawUninstallCleanupItemsInput {
  os: 'windows' | 'macos' | 'linux'
  removeConfigDir: boolean
}

export const OPENCLAW_UNINSTALL_CONFIRM_PHRASE = '我确定卸载 OpenClaw'

export const shouldShowOpenClawUninstallAction = ({
  envMode,
}: OpenClawUninstallActionVisibilityInput): boolean => envMode === 'local'

export const resolveOpenClawUninstallActionState = ({
  envMode,
  openclawInstalled,
  loading,
}: OpenClawUninstallActionStateInput) => {
  if (loading) {
    return {
      disabled: true,
      reason: '正在卸载 OpenClaw...',
    }
  }

  if (envMode !== 'local') {
    return {
      disabled: true,
      reason: 'SSH 环境不支持直接卸载本机 OpenClaw',
    }
  }

  if (!openclawInstalled) {
    return {
      disabled: true,
      reason: '当前本机未检测到 OpenClaw 安装',
    }
  }

  return {
    disabled: false,
    reason: '',
  }
}

export const resolveOpenClawUninstallCleanupItems = ({
  os,
  removeConfigDir,
}: OpenClawUninstallCleanupItemsInput): string[] => {
  const items = [
    os === 'windows'
      ? '停止并删除 openclaw-gateway Windows 服务（NSSM）'
      : '停止并卸载 OpenClaw Gateway 服务',
    '清理当前应用托管的 OpenClaw 运行入口',
  ]

  if (removeConfigDir) {
    items.push('删除 ~/.openclaw 目录中的配置、工作区、缓存与日志')
    items.push('清理用户 PATH 中的托管运行时配置')
    return items
  }

  items.push('保留 ~/.openclaw 中的配置、工作区和运行时，方便后续重新安装')
  return items
}

export const canConfirmOpenClawUninstallPhrase = (value: string): boolean =>
  value.trim() === OPENCLAW_UNINSTALL_CONFIRM_PHRASE
