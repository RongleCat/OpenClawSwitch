// OpenClaw 配置类型定义

/** 模型成本配置 */
export interface CostConfig {
  input?: number
  output?: number
  cacheRead?: number
  cacheWrite?: number
}

/** 单个模型配置 */
export interface ModelConfig {
  id: string
  name?: string
  reasoning?: boolean
  input?: string[]
  cost?: CostConfig
  contextWindow?: number
  maxTokens?: number
  headers?: Record<string, string>
  compat?: Record<string, unknown>
}

/** 提供商配置 */
export interface ProviderConfig {
  baseUrl: string
  apiKey?: string
  api?: string
  models?: ModelConfig[]
}

/** 模型选择配置 */
export interface ModelSelection {
  primary: string
  fallbacks?: string[]
}

/** agents.defaults 配置 */
export interface AgentDefaults {
  model?: ModelSelection
  models?: Record<string, { alias?: string }>
  workspace?: string
  compaction?: { mode?: string }
  maxConcurrent?: number
  subagents?: { maxConcurrent?: number }
}

/** 完整的 OpenClaw 配置（使用 any 保留未知字段） */
export interface OpenClawConfig {
  meta?: {
    lastTouchedVersion?: string
    lastTouchedAt?: string
  }
  models?: {
    mode?: string
    providers?: Record<string, ProviderConfig>
  }
  agents?: {
    defaults?: AgentDefaults
    list?: Array<{ model?: string }>
  }
  // 其他字段作为透传
  [key: string]: unknown
}

/** 返回给前端的模型信息 */
export interface ModelInfo {
  id: string
  name?: string
  reasoning: boolean
  contextWindow?: number
}

/** 返回给前端的提供商信息 */
export interface ProviderInfo {
  name: string
  baseUrl: string
  hasApiKey: boolean
  api?: string
  modelCount: number
  models: ModelInfo[]
}

/** 返回给前端的模型选择信息 */
export interface ModelSelectionInfo {
  primary: string | null
  fallbacks: string[]
}

/** 配置文件信息 */
export interface ConfigFileInfo {
  path: string
  mode: 'local' | 'remote' | 'ssh'
  fileName: string
  dirPath: string
}

/** 文件操作模式 */
export type FileMode = 'local' | 'remote' | 'ssh'

/** 保存模式 */
export type SaveMode = 'overwrite' | 'saveAs'

/** 提供商预设配置 */
export interface ProviderPreset {
  name: string
  displayName: string
  baseUrl: string
}

// ============================================================================
// SSH 相关类型
// ============================================================================

/** SSH 认证方式 */
export type SshAuthMode = 'password' | 'privateKey'

/** SSH 连接配置 */
export interface SshProfile {
  id: string
  name: string
  host: string
  port: number
  username: string
  authMode: SshAuthMode
  password?: string
  keyPath?: string
}

/** SSH 指纹信息 */
export interface FingerprintInfo {
  sha256: string
  md5: string
  host: string
  isKnown: boolean
}

/** 远程文件条目 */
export interface RemoteFileEntry {
  name: string
  path: string
  isDir: boolean
  size: number
}

/** 配置文件搜索结果 */
export interface ConfigSearchResult {
  path: string
  fileName: string
  dirPath: string
}

// ============================================================================
// 安装管理相关类型
// ============================================================================

/** OpenClaw 安装状态 */
export interface OpenClawStatus {
  installed: boolean
  version: string | null
  path: string | null
}

/** Node.js 安装状态 */
export interface NodeStatus {
  installed: boolean
  version: string | null
  meetsRequirement: boolean
}

/** Git 安装状态 */
export interface GitStatusInfo {
  installed: boolean
  version: string | null
}

/** fnm 安装状态 */
export interface FnmStatus {
  installed: boolean
  version: string | null
}

/** 系统信息 */
export interface SystemInfo {
  os: 'windows' | 'macos' | 'linux'
  arch: 'x86_64' | 'aarch64'
  shell: string
}

/** 环境检测综合结果 */
export interface EnvironmentStatus {
  openclaw: OpenClawStatus
  node: NodeStatus
  git: GitStatusInfo
  fnm: FnmStatus
  system: SystemInfo
  networkRegion: string
}

/** V2 应用状态机输入快照 */
export interface AppStateSnapshot {
  envConnected: boolean
  openclawInstalled: boolean
  configLoaded: boolean
  primaryModelValid: boolean
  gatewayReachable: boolean
  lastActionFailed: boolean
}

/** 安装日志事件 */
export interface InstallLogEvent {
  step: string
  message: string
  level: 'info' | 'warn' | 'error' | 'success'
  timestamp: number
}

/** 安装进度事件 */
export interface InstallProgressEvent {
  currentStep: number
  totalSteps: number
  stepName: string
  status: 'running' | 'success' | 'error'
}

/** 下载进度事件 */
export interface InstallDownloadEvent {
  step: string
  percent: number
  speed: string
  downloaded: number
  total: number
}

/** 步骤耗时事件 */
export interface InstallStepTimingEvent {
  step: string
  startTime: number
  endTime: number
  duration: number
}

// ============================================================================
// 导航相关类型
// ============================================================================

/** 页面 ID */
export type PageId = 'home' | 'install' | 'config' | 'ssh' | 'tools'
