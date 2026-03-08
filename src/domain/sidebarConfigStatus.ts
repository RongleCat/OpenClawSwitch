export interface SidebarConfigStatusActionInput {
  envMode: 'local' | 'ssh'
  configStatusText: string
  configFilePath: string
}

export const shouldShowOpenConfigFileAction = (
  input: SidebarConfigStatusActionInput
): boolean =>
  input.envMode === 'local' &&
  input.configStatusText === '配置有效' &&
  input.configFilePath.trim().length > 0
