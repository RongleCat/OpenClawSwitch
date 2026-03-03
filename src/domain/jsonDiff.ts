export interface JsonDiffSummary {
  changed: number
  added: number
  removed: number
  changedKeys: string[]
  oldSize: number
  newSize: number
}

function toPlainObject(input: unknown): Record<string, unknown> {
  if (!input || typeof input !== 'object' || Array.isArray(input)) {
    return {}
  }
  return input as Record<string, unknown>
}

export function buildJsonDiffSummary(
  oldObj: unknown,
  newObj: unknown
): JsonDiffSummary {
  const oldMap = toPlainObject(oldObj)
  const newMap = toPlainObject(newObj)
  const keys = new Set([...Object.keys(oldMap), ...Object.keys(newMap)])

  let changed = 0
  let added = 0
  let removed = 0
  const changedKeys: string[] = []

  for (const key of keys) {
    const oldHas = Object.prototype.hasOwnProperty.call(oldMap, key)
    const newHas = Object.prototype.hasOwnProperty.call(newMap, key)

    if (!oldHas && newHas) {
      added += 1
      changed += 1
      changedKeys.push(key)
      continue
    }

    if (oldHas && !newHas) {
      removed += 1
      changed += 1
      changedKeys.push(key)
      continue
    }

    const oldValue = JSON.stringify(oldMap[key])
    const newValue = JSON.stringify(newMap[key])
    if (oldValue !== newValue) {
      changed += 1
      changedKeys.push(key)
    }
  }

  const oldStr = JSON.stringify(oldObj)
  const newStr = JSON.stringify(newObj)

  return {
    changed,
    added,
    removed,
    changedKeys,
    oldSize: oldStr?.length || 0,
    newSize: newStr?.length || 0,
  }
}
