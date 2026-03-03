export interface DoctorLogLine {
  message: string
  level: 'info' | 'warn' | 'error' | 'success'
  timestamp: number
}

export interface DoctorIssue {
  id: string
  level: 'error' | 'warn'
  message: string
  context: string
  timestamp: number
  sourceLine: number
}

const ERROR_PATTERNS: ReadonlyArray<RegExp> = [
  /\berror\b/i,
  /\bfatal\b/i,
  /\bfail(?:ed|ure)?\b/i,
  /\bexception\b/i,
  /\bpanic\b/i,
  /\bdenied\b/i,
  /\bunauthorized\b/i,
  /\bforbidden\b/i,
  /\brefused\b/i,
  /\bunreachable\b/i,
  /\btimeout\b/i,
  /\btimed out\b/i,
  /\beconn\w*\b/i,
  /\benoent\b/i,
  /\beacces\b/i,
  /not found/i,
  /invalid/i,
  /cannot/i,
  /unable to/i,
  /错误/,
  /失败/,
  /异常/,
  /未找到/,
  /无法/,
  /超时/,
  /✗/,
  /×/,
]

const WARN_PATTERNS: ReadonlyArray<RegExp> = [
  /\bwarn(?:ing)?\b/i,
  /deprecated/i,
  /\brisk\b/i,
  /注意/,
  /警告/,
  /建议/,
]

const IGNORE_PATTERNS: ReadonlyArray<RegExp> = [
  /\b0 errors?\b/i,
  /\berrors?\s*:\s*0\b/i,
  /\bno errors?\b/i,
  /\bwithout errors?\b/i,
  /\b0 warnings?\b/i,
  /no channel security warnings detected/i,
  /\ball checks passed\b/i,
  /\bcompleted successfully\b/i,
]

function sanitizeIssueMessage(line: string): string {
  return line
    .replace(/^\s*\d{1,2}:\d{2}:\d{2}(?:\.\d+)?\s+/, '')
    .replace(/^\s*(info|warn|warning|error|debug|trace)\s+/i, '')
    .trim()
}

function normalizeIssueKey(line: string): string {
  return line
    .toLowerCase()
    .replace(/['"`]/g, '')
    .replace(/\b\d+\b/g, '<n>')
    .replace(/\s+/g, ' ')
    .trim()
}

function getLineContext(lines: DoctorLogLine[], index: number): string {
  const start = Math.max(0, index - 1)
  const end = Math.min(lines.length - 1, index + 1)
  const context = lines
    .slice(start, end + 1)
    .map((item) => item.message.trim())
    .filter(Boolean)
  return context.join('\n')
}

function matchSeverity(line: DoctorLogLine): 'error' | 'warn' | null {
  const message = line.message.trim()
  if (!message) return null

  if (IGNORE_PATTERNS.some((pattern) => pattern.test(message))) {
    return null
  }

  const hasErrorSignal =
    line.level === 'error' || ERROR_PATTERNS.some((pattern) => pattern.test(message))
  if (hasErrorSignal) return 'error'

  const hasWarnSignal = WARN_PATTERNS.some((pattern) => pattern.test(message))
  return hasWarnSignal ? 'warn' : null
}

export function extractDoctorIssues(
  lines: DoctorLogLine[],
  maxIssues = 12
): DoctorIssue[] {
  const issues: DoctorIssue[] = []
  const seen = new Set<string>()

  for (let index = 0; index < lines.length; index += 1) {
    const current = lines[index]
    const level = matchSeverity(current)
    if (!level) continue

    const message = sanitizeIssueMessage(current.message)
    const dedupeKey = `${level}:${normalizeIssueKey(message)}`
    if (seen.has(dedupeKey)) continue
    seen.add(dedupeKey)

    issues.push({
      id: dedupeKey,
      level,
      message,
      context: getLineContext(lines, index),
      timestamp: current.timestamp,
      sourceLine: index + 1,
    })

    if (issues.length >= maxIssues) break
  }

  return issues
}
