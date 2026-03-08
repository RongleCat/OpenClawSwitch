import { describe, expect, it } from 'vitest'
import { formatGatewayInstallError, isAdminRequiredGatewayInstallError } from './postInstallError'

describe('formatGatewayInstallError', () => {
  it('adds actionable hint when nssm is missing', () => {
    const input = '网关服务安装失败: 未检测到 nssm，请先安装 nssm 并确保 nssm 在 PATH 中可用'
    const output = formatGatewayInstallError(input)
    expect(output).toContain('nssm 不是 Windows 系统自带组件')
  })

  it('keeps generic errors readable', () => {
    const output = formatGatewayInstallError('网关服务安装失败: 权限不足')
    expect(output).toContain('权限不足')
  })

  it('adds admin hint when service access is denied', () => {
    const input = "网关服务安装失败: Can't open service! OpenService(): 拒绝访问。"
    const output = formatGatewayInstallError(input)
    expect(output).toContain('请以管理员身份运行')
  })

  it('adds conflict hint when same-name service is not managed by nssm', () => {
    const input = '网关服务安装失败: 检测到已存在同名 Windows 服务 openclaw-gateway，但它不是由 OpenClawSwitch 管理'
    const output = formatGatewayInstallError(input)
    expect(output).toContain('同名 Windows 服务')
    expect(output).toContain('手动处理')
  })

  it('detects admin-required gateway install errors', () => {
    expect(
      isAdminRequiredGatewayInstallError("网关服务安装失败: Can't open service! OpenService(): 拒绝访问。")
    ).toBe(true)
    expect(
      isAdminRequiredGatewayInstallError('网关服务安装失败: 未检测到 nssm，请先安装 nssm 并确保 nssm 在 PATH 中可用')
    ).toBe(false)
  })
})
