const NSSM_MISSING_HINT =
  'nssm 不是 Windows 系统自带组件。已自动尝试 winget/choco/scoop/直连下载；如仍失败，请将 nssm.exe 放到 ~/.openclaw/bin/nssm/nssm.exe 后重试。'
const ADMIN_REQUIRED_HINT =
  '这一步需要管理员权限。请以管理员身份运行 OpenClawSwitch（可右键选择“以管理员身份运行”）后重试，或改用“手动配置”。'
const SERVICE_CONFLICT_HINT =
  '系统里已经有同名 Windows 服务，但不是当前应用托管的服务。请先手动处理该服务，再重试默认配置。'

export const isAdminRequiredGatewayInstallError = (error: unknown) => {
  const raw = String(error ?? '')
  return (
    raw.includes('管理员权限') ||
    raw.includes('拒绝访问') ||
    raw.includes("Can't open service!") ||
    raw.toLowerCase().includes('access is denied')
  )
}

export const formatGatewayInstallError = (error: unknown) => {
  const raw = String(error ?? '')
  if (raw.includes('未检测到 nssm')) {
    return `${raw}\n${NSSM_MISSING_HINT}`
  }
  if (isAdminRequiredGatewayInstallError(raw)) {
    return `${raw}\n${ADMIN_REQUIRED_HINT}`
  }
  if (raw.includes('已存在同名 Windows 服务') || raw.includes('不是由 OpenClawSwitch/nssm 管理')) {
    return `${raw}\n${SERVICE_CONFLICT_HINT}`
  }
  return raw
}
