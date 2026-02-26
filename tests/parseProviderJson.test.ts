import { describe, it, expect } from 'vitest'
import { parseProviderJson } from '../src/utils/parseProviderJson'

// 用户提供的完整测试用例
const fullNestedJson = `"models": {
  "mode": "merge",
  "providers": {
    "bailian": {
      "baseUrl": "https://coding.dashscope.aliyuncs.com/v1",
      "apiKey": "YOUR_API_KEY",
      "api": "openai-completions",
      "models": [
        {
          "id": "qwen3.5-plus",
          "name": "qwen3.5-plus",
          "reasoning": false,
          "input": ["text", "image"],
          "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 },
          "contextWindow": 1000000,
          "maxTokens": 65536
        },
        {
          "id": "qwen3-max-2026-01-23",
          "name": "qwen3-max-2026-01-23",
          "reasoning": false,
          "input": ["text"],
          "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 },
          "contextWindow": 262144,
          "maxTokens": 65536
        },
        {
          "id": "qwen3-coder-next",
          "name": "qwen3-coder-next",
          "reasoning": false,
          "input": ["text"],
          "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 },
          "contextWindow": 262144,
          "maxTokens": 65536
        },
        {
          "id": "qwen3-coder-plus",
          "name": "qwen3-coder-plus",
          "reasoning": false,
          "input": ["text"],
          "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 },
          "contextWindow": 1000000,
          "maxTokens": 65536
        },
        {
          "id": "MiniMax-M2.5",
          "name": "MiniMax-M2.5",
          "reasoning": false,
          "input": ["text"],
          "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 },
          "contextWindow": 1000000,
          "maxTokens": 65536
        },
        {
          "id": "glm-5",
          "name": "glm-5",
          "reasoning": false,
          "input": ["text"],
          "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 },
          "contextWindow": 202752,
          "maxTokens": 16384
        },
        {
          "id": "glm-4.7",
          "name": "glm-4.7",
          "reasoning": false,
          "input": ["text"],
          "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 },
          "contextWindow": 202752,
          "maxTokens": 16384
        },
        {
          "id": "kimi-k2.5",
          "name": "kimi-k2.5",
          "reasoning": false,
          "input": ["text", "image"],
          "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 },
          "contextWindow": 262144,
          "maxTokens": 32768
        }
      ]
    }
  }
}`

describe('parseProviderJson', () => {
  it('解析用户提供的完整嵌套 JSON（非标准，以 "models": 开头）', () => {
    const result = parseProviderJson(fullNestedJson)

    expect(result.error).toBe('')
    expect(result.provider).not.toBeNull()
    expect(result.name).toBe('bailian')
    expect(result.provider!.baseUrl).toBe('https://coding.dashscope.aliyuncs.com/v1')
    expect(result.provider!.apiKey).toBe('YOUR_API_KEY')
    expect(result.provider!.api).toBe('openai-completions')
    expect(result.provider!.models).toHaveLength(8)
    expect(result.provider!.models![0].id).toBe('qwen3.5-plus')
    expect(result.provider!.models![7].id).toBe('kimi-k2.5')
  })

  it('解析标准 JSON 包裹的 models.providers 结构', () => {
    const json = JSON.stringify({
      models: {
        mode: 'merge',
        providers: {
          myProvider: {
            baseUrl: 'https://api.example.com/v1',
            apiKey: 'sk-test',
            models: [{ id: 'model-1', name: 'Model 1' }]
          }
        }
      }
    })
    const result = parseProviderJson(json)

    expect(result.error).toBe('')
    expect(result.name).toBe('myProvider')
    expect(result.provider!.baseUrl).toBe('https://api.example.com/v1')
    expect(result.provider!.models).toHaveLength(1)
  })

  it('解析直接的 provider 配置', () => {
    const json = JSON.stringify({
      baseUrl: 'https://api.openai.com/v1',
      apiKey: 'sk-xxx',
      models: [{ id: 'gpt-4', name: 'GPT-4' }]
    })
    const result = parseProviderJson(json)

    expect(result.error).toBe('')
    expect(result.name).toBe('')
    expect(result.provider!.baseUrl).toBe('https://api.openai.com/v1')
  })

  it('解析 providers 层级（无 models 包裹）', () => {
    const json = JSON.stringify({
      providers: {
        deepseek: {
          baseUrl: 'https://api.deepseek.com',
          models: [{ id: 'deepseek-chat' }]
        }
      }
    })
    const result = parseProviderJson(json)

    expect(result.error).toBe('')
    expect(result.name).toBe('deepseek')
    expect(result.provider!.baseUrl).toBe('https://api.deepseek.com')
  })

  it('空字符串返回空结果', () => {
    const result = parseProviderJson('')
    expect(result.provider).toBeNull()
    expect(result.error).toBe('')
  })

  it('无效 JSON 返回错误', () => {
    const result = parseProviderJson('not json at all')
    expect(result.provider).toBeNull()
    expect(result.error).toContain('JSON 格式无效')
  })

  it('缺少 baseUrl 返回错误', () => {
    const json = JSON.stringify({ models: [{ id: 'test' }] })
    const result = parseProviderJson(json)
    expect(result.provider).toBeNull()
    expect(result.error).toContain('无法识别')
  })

  it('解析以 "providers": 开头的非标准 JSON 片段', () => {
    const text = `"providers": {
    "bailian": {
      "baseUrl": "https://coding.dashscope.aliyuncs.com/v1",
      "apiKey": "YOUR_API_KEY",
      "api": "openai-completions",
      "models": [
        {
          "id": "qwen3.5-plus",
          "name": "qwen3.5-plus",
          "reasoning": false,
          "input": ["text", "image"],
          "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 },
          "contextWindow": 1000000,
          "maxTokens": 65536
        },
        {
          "id": "kimi-k2.5",
          "name": "kimi-k2.5",
          "reasoning": false,
          "input": ["text", "image"],
          "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 },
          "contextWindow": 262144,
          "maxTokens": 32768
        }
      ]
    }
  }`
    const result = parseProviderJson(text)

    expect(result.error).toBe('')
    expect(result.provider).not.toBeNull()
    expect(result.name).toBe('bailian')
    expect(result.provider!.baseUrl).toBe('https://coding.dashscope.aliyuncs.com/v1')
    expect(result.provider!.models).toHaveLength(2)
  })

  it('解析以 "name": { provider } 开头的非标准 JSON 片段', () => {
    const text = `"bailian": {
      "baseUrl": "https://coding.dashscope.aliyuncs.com/v1",
      "apiKey": "YOUR_API_KEY",
      "api": "openai-completions",
      "models": [
        {
          "id": "qwen3.5-plus",
          "name": "qwen3.5-plus",
          "reasoning": false,
          "input": ["text", "image"],
          "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 },
          "contextWindow": 1000000,
          "maxTokens": 65536
        },
        {
          "id": "kimi-k2.5",
          "name": "kimi-k2.5",
          "reasoning": false,
          "input": ["text", "image"],
          "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 },
          "contextWindow": 262144,
          "maxTokens": 32768
        }
      ]
    }`
    const result = parseProviderJson(text)

    expect(result.error).toBe('')
    expect(result.provider).not.toBeNull()
    expect(result.name).toBe('bailian')
    expect(result.provider!.baseUrl).toBe('https://coding.dashscope.aliyuncs.com/v1')
    expect(result.provider!.models).toHaveLength(2)
  })

  it('解析直接的完整 provider JSON 对象（无名称包裹）', () => {
    const text = `{
      "baseUrl": "https://coding.dashscope.aliyuncs.com/v1",
      "apiKey": "YOUR_API_KEY",
      "api": "openai-completions",
      "models": [
        {
          "id": "qwen3.5-plus",
          "name": "qwen3.5-plus",
          "reasoning": false,
          "input": ["text", "image"],
          "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 },
          "contextWindow": 1000000,
          "maxTokens": 65536
        },
        {
          "id": "kimi-k2.5",
          "name": "kimi-k2.5",
          "reasoning": false,
          "input": ["text", "image"],
          "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 },
          "contextWindow": 262144,
          "maxTokens": 32768
        }
      ]
    }`
    const result = parseProviderJson(text)

    expect(result.error).toBe('')
    expect(result.provider).not.toBeNull()
    expect(result.name).toBe('')  // 无名称包裹，名称为空
    expect(result.provider!.baseUrl).toBe('https://coding.dashscope.aliyuncs.com/v1')
    expect(result.provider!.api).toBe('openai-completions')
    expect(result.provider!.models).toHaveLength(2)
  })
})
