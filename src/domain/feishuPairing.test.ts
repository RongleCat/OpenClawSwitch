import { describe, expect, it } from 'vitest'
import {
  FEISHU_PAIRING_APPROVE_COMMAND_EXAMPLE,
  extractFeishuPairingCode,
} from './feishuPairing'

describe('FEISHU_PAIRING_APPROVE_COMMAND_EXAMPLE', () => {
  it('documents the notify-enabled official approve command', () => {
    expect(FEISHU_PAIRING_APPROVE_COMMAND_EXAMPLE).toBe(
      'openclaw pairing approve feishu UZM4NXNC --notify'
    )
  })
})

describe('extractFeishuPairingCode', () => {
  it('extracts the pairing code from the official approve command with notify flag', () => {
    expect(
      extractFeishuPairingCode('openclaw pairing approve feishu UZM4NXNC --notify')
    ).toBe('UZM4NXNC')
  })

  it('accepts a raw pairing code and trims shell quotes', () => {
    expect(extractFeishuPairingCode('  "UZM4NXNC"  ')).toBe('UZM4NXNC')
  })
})
