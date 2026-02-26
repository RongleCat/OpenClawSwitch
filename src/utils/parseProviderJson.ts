import type { ProviderConfig } from '../types/config'

export interface ParseResult {
  provider: ProviderConfig | null
  name: string
  error: string
}

/**
 * 智能解析粘贴的服务商 JSON 配置
 * 支持多种嵌套格式：
 * 1. 直接 provider: { baseUrl, models[] }
 * 2. { providers: { name: { baseUrl, models[] } } }
 * 3. { models: { providers: { name: { baseUrl, models[] } } } }
 * 4. { name: { baseUrl, models[] } } — 兜底：对象的 value 是 provider
 */
export function parseProviderJson(text: string): ParseResult {
  const empty: ParseResult = { provider: null, name: '', error: '' }

  if (!text.trim()) return empty

  let parsed: any = null
  let prefixName = '' // 从非标准 JSON 前缀提取的名称

  // 尝试直接解析
  try {
    parsed = JSON.parse(text.trim())
  } catch {
    // 尝试提取最外层 {...}
    const match = text.match(/\{[\s\S]*\}/)
    if (match) {
      try {
        parsed = JSON.parse(match[0])
        // 从 {...} 前面的文本中提取 key 名称，如 "bailian": { 或 "providers": {
        const prefix = text.slice(0, text.indexOf(match[0]))
        const nameMatch = prefix.match(/"([^"]+)"\s*:\s*$/)
        if (nameMatch) {
          prefixName = nameMatch[1]
        }
      } catch {
        return { ...empty, error: 'JSON 格式无效，请检查配置内容' }
      }
    } else {
      return { ...empty, error: 'JSON 格式无效，请检查配置内容' }
    }
  }

  // 智能提取核心 provider 配置
  let providerData: any = null
  let extractedName = ''

  if (parsed.baseUrl && Array.isArray(parsed.models)) {
    providerData = parsed
  } else if (parsed.providers && typeof parsed.providers === 'object') {
    const keys = Object.keys(parsed.providers)
    if (keys.length > 0) {
      extractedName = keys[0]
      providerData = parsed.providers[keys[0]]
    }
  } else if (parsed.models && typeof parsed.models === 'object' && !Array.isArray(parsed.models)) {
    const modelsObj = parsed.models
    if (modelsObj.providers && typeof modelsObj.providers === 'object') {
      const keys = Object.keys(modelsObj.providers)
      if (keys.length > 0) {
        extractedName = keys[0]
        providerData = modelsObj.providers[keys[0]]
      }
    }
  }

  // 兜底：对象的 value 本身就是 provider 配置（如 { "bailian": { baseUrl, models } }）
  if (!providerData) {
    const keys = Object.keys(parsed)
    for (const key of keys) {
      const val = parsed[key]
      if (val && typeof val === 'object' && val.baseUrl) {
        extractedName = key
        providerData = val
        break
      }
    }
  }

  if (!providerData || !providerData.baseUrl) {
    return { ...empty, error: '无法识别有效的服务商配置，需包含 baseUrl 和 models' }
  }

  return {
    provider: providerData as ProviderConfig,
    name: extractedName || prefixName,
    error: ''
  }
}
