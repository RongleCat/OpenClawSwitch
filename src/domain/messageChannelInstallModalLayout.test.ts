import { describe, expect, it } from 'vitest'
import { messageChannelInstallModalLayout } from './messageChannelInstallModalLayout'

describe('messageChannelInstallModal layout', () => {
  it('locks dialog overflow and keeps scrolling inside the log viewport', () => {
    expect(messageChannelInstallModalLayout.card).toContain('oc-modal-card')
    expect(messageChannelInstallModalLayout.card).toContain('oc-channel-install-modal')
    expect(messageChannelInstallModalLayout.card).toContain('max-h-[82vh]')
    expect(messageChannelInstallModalLayout.card).toContain('flex')
    expect(messageChannelInstallModalLayout.card).toContain('flex-col')
    expect(messageChannelInstallModalLayout.logViewport).toContain('oc-channel-install-log-viewport')
    expect(messageChannelInstallModalLayout.logViewport).toContain('min-h-0')
    expect(messageChannelInstallModalLayout.logViewport).toContain('flex-1')
  })
})
