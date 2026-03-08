export const FEISHU_PAIRING_APPROVE_COMMAND_EXAMPLE =
  'openclaw pairing approve feishu UZM4NXNC --notify'

export const extractFeishuPairingCode = (raw: string): string => {
  const compact = raw.trim().replace(/\s+/g, ' ')
  if (!compact) return ''

  const commandMatch = compact.match(/^openclaw\s+pairing\s+approve\s+feishu\s+(.+)$/i)
  const commandCode = commandMatch ? commandMatch[1] : ''

  const candidate = commandCode ? commandCode.trim() : compact
  const firstToken = candidate.split(/\s+/)[0] || ''
  return firstToken.replace(/^['"]|['"]$/g, '').trim()
}
