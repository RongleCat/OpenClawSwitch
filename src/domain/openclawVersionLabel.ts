const OPENCLAW_DATE_VERSION_PATTERN = /\b\d{4}\.\d{1,2}\.\d{1,2}\b/

export const formatOpenClawVersionLabel = (version: string | null | undefined): string => {
  const raw = version?.trim()
  if (!raw) return '--'

  const matchedDate = raw.match(OPENCLAW_DATE_VERSION_PATTERN)
  if (matchedDate) {
    return matchedDate[0]
  }

  return raw
}
