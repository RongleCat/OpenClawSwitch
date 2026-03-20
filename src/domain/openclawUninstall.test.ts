import { describe, expect, it } from 'vitest'
import {
  OPENCLAW_UNINSTALL_CONFIRM_PHRASE,
  canConfirmOpenClawUninstallPhrase,
  resolveOpenClawUninstallActionState,
  resolveOpenClawUninstallCleanupItems,
  shouldShowOpenClawUninstallAction,
} from './openclawUninstall'

describe('shouldShowOpenClawUninstallAction', () => {
  it('shows uninstall action only in local mode', () => {
    expect(shouldShowOpenClawUninstallAction({ envMode: 'local' })).toBe(true)
    expect(shouldShowOpenClawUninstallAction({ envMode: 'ssh' })).toBe(false)
  })
})

describe('resolveOpenClawUninstallActionState', () => {
  it('keeps the button enabled for installed local OpenClaw', () => {
    expect(
      resolveOpenClawUninstallActionState({
        envMode: 'local',
        openclawInstalled: true,
        loading: false,
      })
    ).toEqual({
      disabled: false,
      reason: '',
    })
  })

  it('disables the button when local OpenClaw is not installed', () => {
    expect(
      resolveOpenClawUninstallActionState({
        envMode: 'local',
        openclawInstalled: false,
        loading: false,
      })
    ).toEqual({
      disabled: true,
      reason: '当前本机未检测到 OpenClaw 安装',
    })
  })

  it('disables the button in ssh mode even if a remote environment is connected', () => {
    expect(
      resolveOpenClawUninstallActionState({
        envMode: 'ssh',
        openclawInstalled: true,
        loading: false,
      })
    ).toEqual({
      disabled: true,
      reason: 'SSH 环境不支持直接卸载本机 OpenClaw',
    })
  })

  it('disables the button while uninstall is running', () => {
    expect(
      resolveOpenClawUninstallActionState({
        envMode: 'local',
        openclawInstalled: true,
        loading: true,
      })
    ).toEqual({
      disabled: true,
      reason: '正在卸载 OpenClaw...',
    })
  })
})

describe('canConfirmOpenClawUninstallPhrase', () => {
  it('accepts only the exact confirmation phrase after trimming', () => {
    expect(OPENCLAW_UNINSTALL_CONFIRM_PHRASE).toBe('我确定卸载 OpenClaw')
    expect(canConfirmOpenClawUninstallPhrase(' 我确定卸载 OpenClaw ')).toBe(true)
    expect(canConfirmOpenClawUninstallPhrase('我确定卸载openclaw')).toBe(false)
    expect(canConfirmOpenClawUninstallPhrase('我确定删除 OpenClaw')).toBe(false)
  })
})

describe('resolveOpenClawUninstallCleanupItems', () => {
  it('describes reverse cleanup for windows without deleting config', () => {
    expect(resolveOpenClawUninstallCleanupItems({ os: 'windows', removeConfigDir: false })).toEqual([
      '停止并删除 openclaw-gateway Windows 服务（NSSM）',
      '清理当前应用托管的 OpenClaw 运行入口',
      '保留 ~/.openclaw 中的配置、工作区和运行时，方便后续重新安装',
    ])
  })

  it('adds config and environment cleanup when removing managed directory', () => {
    expect(resolveOpenClawUninstallCleanupItems({ os: 'linux', removeConfigDir: true })).toEqual([
      '停止并卸载 OpenClaw Gateway 服务',
      '清理当前应用托管的 OpenClaw 运行入口',
      '删除 ~/.openclaw 目录中的配置、工作区、缓存与日志',
      '清理用户 PATH 中的托管运行时配置',
    ])
  })
})
