import { describe, expect, it } from 'vitest'
import { messageChannelHeaderLayout } from './messageChannelHeaderLayout'

describe('messageChannelHeaderLayout', () => {
  it('keeps the left column flexible and shrinkable', () => {
    expect(messageChannelHeaderLayout.leftColumn).toContain('min-w-0')
    expect(messageChannelHeaderLayout.leftColumn).toContain('flex-1')
  })

  it('keeps account controls compact and aligned', () => {
    expect(messageChannelHeaderLayout.controlsRow).toContain('items-center')
    expect(messageChannelHeaderLayout.controlsRow).toContain('gap-2')
    expect(messageChannelHeaderLayout.accountSelect).toContain('min-w-[220px]')
    expect(messageChannelHeaderLayout.accountSelectTrigger).toContain('w-[232px]')
    expect(messageChannelHeaderLayout.accountSelectTrigger).toContain('min-w-[208px]')
    expect(messageChannelHeaderLayout.accountSelectTrigger).toContain('!h-8')
    expect(messageChannelHeaderLayout.accountSelectMenu).toContain('oc-dropdown-menu')
    expect(messageChannelHeaderLayout.accountButton).toContain('h-8')
  })

  it('prevents header actions from shrinking or wrapping', () => {
    expect(messageChannelHeaderLayout.rightAction).toContain('shrink-0')
    expect(messageChannelHeaderLayout.rightAction).toContain('whitespace-nowrap')
    expect(messageChannelHeaderLayout.toggleWrap).toContain('shrink-0')
    expect(messageChannelHeaderLayout.toggleControl).toContain('oc-channel-toggle')
    expect(messageChannelHeaderLayout.toggleThumb).toContain('oc-channel-toggle-thumb')
  })

  it('keeps icons from compressing', () => {
    expect(messageChannelHeaderLayout.icon).toContain('shrink-0')
  })
})
