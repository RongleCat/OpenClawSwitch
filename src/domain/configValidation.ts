export function isPrimaryModelPlaceholder(model: string | null | undefined): boolean {
  if (!model) return true
  const normalized = model.trim()
  return normalized === '' || normalized === 'local/placeholder'
}
