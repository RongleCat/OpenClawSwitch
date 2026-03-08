import { describe, expect, it } from 'vitest'
import {
  CONFIG_PAGE_DESCRIPTION,
  resolveConfigPagePrimaryActionState,
} from './configPageToolbar'

describe('configPageToolbar', () => {
  it('uses save-only description', () => {
    expect(CONFIG_PAGE_DESCRIPTION).toBe('选择配置文件、调整模型并保存。')
    expect(CONFIG_PAGE_DESCRIPTION.includes('验证')).toBe(false)
  })

  it('uses save-only primary action label', () => {
    expect(
      resolveConfigPagePrimaryActionState({
        canSave: true,
        loading: false,
        primaryModelInvalid: false,
      })
    ).toEqual({
      show: true,
      label: '保存配置',
      disabled: false,
    })
  })

  it('disables save action when saving is unavailable', () => {
    expect(
      resolveConfigPagePrimaryActionState({
        canSave: true,
        loading: true,
        primaryModelInvalid: false,
      })
    ).toEqual({
      show: true,
      label: '保存配置',
      disabled: true,
    })
  })
})
