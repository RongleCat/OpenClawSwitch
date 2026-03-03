export type NavPage =
  | 'overview'
  | 'ai-config'
  | 'diagnostics'
  | 'channels'
  | 'settings'

export interface NavItem {
  id: NavPage
  label: string
  optional?: boolean
}

export const NAV_ITEMS: ReadonlyArray<NavItem> = [
  { id: 'overview', label: '工作台' },
  { id: 'ai-config', label: '模型配置' },
  { id: 'diagnostics', label: '服务诊断' },
  { id: 'channels', label: '消息渠道', optional: true },
  { id: 'settings', label: '系统设置' },
] as const
