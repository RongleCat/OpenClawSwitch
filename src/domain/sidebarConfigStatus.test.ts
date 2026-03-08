import { describe, expect, it } from 'vitest'
import { shouldShowOpenConfigFileAction } from './sidebarConfigStatus'

describe('shouldShowOpenConfigFileAction', () => {
  it('shows open action only for valid local config with a resolved path', () => {
    expect(
      shouldShowOpenConfigFileAction({
        envMode: 'local',
        configStatusText: '配置有效',
        configFilePath: 'C:/Users/test/.openclaw/config.json',
      })
    ).toBe(true)

    expect(
      shouldShowOpenConfigFileAction({
        envMode: 'ssh',
        configStatusText: '配置有效',
        configFilePath: '/home/test/.openclaw/config.json',
      })
    ).toBe(false)

    expect(
      shouldShowOpenConfigFileAction({
        envMode: 'local',
        configStatusText: '主模型无效',
        configFilePath: 'C:/Users/test/.openclaw/config.json',
      })
    ).toBe(false)

    expect(
      shouldShowOpenConfigFileAction({
        envMode: 'local',
        configStatusText: '配置有效',
        configFilePath: '   ',
      })
    ).toBe(false)
  })
})
