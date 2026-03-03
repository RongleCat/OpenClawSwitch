export interface ActivityItem {
  action: string
  status: 'success' | 'error'
  at: number
  detail?: string
}

export function appendActivity(
  list: ActivityItem[],
  item: ActivityItem,
  max = 20
): ActivityItem[] {
  return [item, ...list].slice(0, max)
}
