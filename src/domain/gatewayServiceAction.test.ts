import { describe, expect, it } from 'vitest'
import {
  resolveGatewayQuickActionGridColumns,
  resolveGatewayQuickActionLabel,
  resolveGatewayQuickActionState,
  shouldShowInstallGatewayServiceAction,
} from './gatewayServiceAction'

describe('shouldShowInstallGatewayServiceAction', () => {
  it('shows action only when windows local service is missing', () => {
    expect(
      shouldShowInstallGatewayServiceAction({
        isWindows: true,
        envMode: 'local',
        gatewayServiceInstalled: false,
      })
    ).toBe(true)
  })

  it('hides action when service already installed', () => {
    expect(
      shouldShowInstallGatewayServiceAction({
        isWindows: true,
        envMode: 'local',
        gatewayServiceInstalled: true,
      })
    ).toBe(false)
  })

  it('hides action when gateway is already reachable', () => {
    expect(
      shouldShowInstallGatewayServiceAction({
        isWindows: true,
        envMode: 'local',
        gatewayServiceInstalled: false,
        gatewayReachable: true,
      })
    ).toBe(false)
  })

  it('hides action in ssh mode', () => {
    expect(
      shouldShowInstallGatewayServiceAction({
        isWindows: true,
        envMode: 'ssh',
        gatewayServiceInstalled: false,
      })
    ).toBe(false)
  })

  it('hides action on non-windows systems', () => {
    expect(
      shouldShowInstallGatewayServiceAction({
        isWindows: false,
        envMode: 'local',
        gatewayServiceInstalled: false,
      })
    ).toBe(false)
  })

  it('uses three columns when only three actions are visible', () => {
    expect(resolveGatewayQuickActionGridColumns(3)).toBe(3)
  })

  it('uses four columns when install action is visible', () => {
    expect(resolveGatewayQuickActionGridColumns(4)).toBe(4)
  })

  it('marks pending action as loading and disables all actions during request', () => {
    expect(
      resolveGatewayQuickActionState({
        actionId: 'start',
        baseDisabled: false,
        pendingActionId: 'start',
      })
    ).toEqual({
      loading: true,
      disabled: true,
    })

    expect(
      resolveGatewayQuickActionState({
        actionId: 'restart',
        baseDisabled: false,
        pendingActionId: 'start',
      })
    ).toEqual({
      loading: false,
      disabled: true,
    })
  })

  it('preserves base disabled state when there is no pending action', () => {
    expect(
      resolveGatewayQuickActionState({
        actionId: 'restart',
        baseDisabled: true,
        pendingActionId: null,
      })
    ).toEqual({
      loading: false,
      disabled: true,
    })
  })
})

describe('resolveGatewayQuickActionLabel', () => {
  it('returns loading labels for the pending gateway action', () => {
    expect(resolveGatewayQuickActionLabel('start', '启动', true)).toBe('启动中...')
    expect(resolveGatewayQuickActionLabel('restart', '重启', true)).toBe('重启中...')
    expect(resolveGatewayQuickActionLabel('stop', '停止', true)).toBe('停止中...')
  })

  it('keeps the base label when the action is idle', () => {
    expect(resolveGatewayQuickActionLabel('start', '启动', false)).toBe('启动')
  })
})
