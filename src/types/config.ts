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
  mode: 'local' | 'remote'
  fileName: string
  dirPath: string
}

/** 文件操作模式 */
export type FileMode = 'local' | 'remote'

/** 保存模式 */
export type SaveMode = 'overwrite' | 'saveAs'
