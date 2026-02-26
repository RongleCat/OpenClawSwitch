<script setup lang="ts">
import { ref, computed, onMounted, watch, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { open, save, ask } from '@tauri-apps/api/dialog'
import Button from './components/ui/Button.vue'
import Input from './components/ui/Input.vue'
import Label from './components/ui/Label.vue'
import Card from './components/ui/Card.vue'
import Toast from './components/ui/Toast.vue'
import ProviderCard from './components/ProviderCard.vue'
import SshConnectModal from './components/SshConnectModal.vue'
import SshFingerprintDialog from './components/SshFingerprintDialog.vue'
import RemoteFileBrowser from './components/RemoteFileBrowser.vue'
import {
  Server, Settings, ListTree, Save, Download, Plus, X, ChevronDown, FolderOpen, FileCode,
  RefreshCw, Terminal, Wrench, Hammer, Monitor, WifiOff
} from 'lucide-vue-next'
import type {
  OpenClawConfig, ProviderInfo, ModelSelectionInfo, ConfigFileInfo, ProviderPreset,
  FingerprintInfo
} from './types/config'

// ============================================================================
// 状态管理
// ============================================================================

// 配置状态
const currentConfig = ref<OpenClawConfig | null>(null)
const fileInfo = ref<ConfigFileInfo | null>(null)
const isDirty = ref(false)
const lastSaveTime = ref<string | null>(null)

// 提供商和模型状态
const providers = ref<ProviderInfo[]>([])
const modelSelection = ref<ModelSelectionInfo>({ primary: null, fallbacks: [] })

// UI 状态
const loading = ref(false)

// SSH 状态
const showSshModal = ref(false)
const showFingerprintDialog = ref(false)
const showFileBrowser = ref(false)
const sshConnected = ref(false)
const sshFingerprint = ref<FingerprintInfo | null>(null)
const sshFingerprintCallback = ref<(() => void) | null>(null)
const sshRemotePath = ref('')

// 弹窗状态
const showProviderModal = ref(false)
const showModelModal = ref(false)
const modelModalProvider = ref('')
const showSourceModal = ref(false)

// 主模型选择下拉
const showPrimarySelector = ref(false)

// 新提供商表单
const newProvider = ref({
  name: '',
  baseUrl: '',
  apiKey: ''
})

// 粘贴配置模式
const providerModalTab = ref<'manual' | 'paste'>('manual')
const pasteJsonText = ref('')
const pasteProviderName = ref('')
const pasteApiKey = ref('')
const pasteParseError = ref('')
const parsedProviderConfig = ref<import('./types/config').ProviderConfig | null>(null)

// 编辑态
const isEditingProvider = ref(false)
const editingProviderName = ref('')

// 新模型表单
const newModelId = ref('')

// 模型列表相关状态
const availableModels = ref<string[]>([])
const loadingModels = ref(false)
const showModelDropdown = ref(false)

// ============================================================================
// 计算属性
// ============================================================================

const isLocalMode = computed(() => fileInfo.value?.mode === 'local')
const isSshMode = computed(() => fileInfo.value?.mode === 'ssh')
const canSave = computed(() => currentConfig.value && fileInfo.value)

// 过滤后的模型列表（支持模糊搜索）
const filteredModels = computed(() => {
  if (!newModelId.value) return availableModels.value
  const search = newModelId.value.toLowerCase()
  return availableModels.value.filter(model =>
    model.toLowerCase().includes(search)
  )
})

// 判断是否存在 Minimax 服务商
const hasMinimaxProvider = computed(() => {
  return currentConfig.value?.models?.providers?.['minimax'] !== undefined
})

// ============================================================================
// 文件操作
// ============================================================================

// 加载默认配置
const loadDefaultConfig = async () => {
  loading.value = true
  try {
    const [config, info] = await invoke<[OpenClawConfig, ConfigFileInfo]>('load_default_config')
    currentConfig.value = config
    fileInfo.value = info
    isDirty.value = false
    await refreshProviders()
    showToast('success', `已加载: ${info.fileName}`)
  } catch (error) {
    console.error('加载默认配置失败:', error)
    showToast('error', `${error}`)
  } finally {
    loading.value = false
  }
}

// 加载本地配置文件
const loadLocalConfig = async () => {
  loading.value = true
  try {
    const [config, info] = await invoke<[OpenClawConfig, ConfigFileInfo]>('load_local_config')
    currentConfig.value = config
    fileInfo.value = info
    isDirty.value = false
    lastSaveTime.value = null
    await refreshProviders()
    showToast('success', `已加载本地配置: ${info.fileName}`)
  } catch (error) {
    console.error('加载本地配置失败:', error)
    showToast('error', `${error}`)
  } finally {
    loading.value = false
  }
}

// 选择配置文件
const selectFile = async () => {
  try {
    const selected = await open({
      filters: [{ name: 'JSON', extensions: ['json'] }],
      title: '选择 OpenClaw 配置文件'
    })
    if (selected && typeof selected === 'string') {
      loading.value = true
      const [config, info] = await invoke<[OpenClawConfig, ConfigFileInfo]>(
        'load_config_from_file',
        { filePath: selected }
      )
      currentConfig.value = config
      fileInfo.value = info
      isDirty.value = false
      await refreshProviders()
      showToast('success', `已加载: ${info.fileName}`)
    }
  } catch (error) {
    console.error('选择文件失败:', error)
    showToast('error', `${error}`)
  } finally {
    loading.value = false
  }
}

// 保存配置
const saveConfig = async () => {
  if (!currentConfig.value || !fileInfo.value) return
  // SSH 模式走专用保存逻辑
  if (isSshMode.value) {
    await saveSshConfig()
    return
  }
  loading.value = true
  try {
    await invoke('save_config', {
      config: currentConfig.value,
      path: fileInfo.value.path
    })
    isDirty.value = false
    lastSaveTime.value = new Date().toLocaleString('zh-CN', { hour12: false })
    showToast('success', '已保存')
  } catch (error) {
    showToast('error', `保存失败: ${error}`)
  } finally {
    loading.value = false
  }
}

// 另存为
const saveConfigAs = async () => {
  if (!currentConfig.value) return
  try {
    const selected = await save({
      filters: [{ name: 'JSON', extensions: ['json'] }],
      defaultPath: 'openclaw.json',
      title: '另存为'
    })
    if (selected) {
      loading.value = true
      await invoke('save_config_as', {
        config: currentConfig.value,
        newPath: selected
      })
      fileInfo.value = {
        path: selected,
        mode: 'remote',
        fileName: selected.split(/[/\\]/).pop() || 'openclaw.json',
        dirPath: selected.substring(0, Math.max(selected.lastIndexOf('/'), selected.lastIndexOf('\\')))
      }
      isDirty.value = false
      showToast('success', `已保存到: ${selected}`)
    }
  } catch (error) {
    showToast('error', `另存为失败: ${error}`)
  } finally {
    loading.value = false
  }
}

// 自动保存（本地模式）
const autoSave = async () => {
  if (isLocalMode.value && isDirty.value) {
    await saveConfig()
  }
}

// ============================================================================
// 配置操作
// ============================================================================

// 刷新提供商列表
const refreshProviders = async () => {
  if (!currentConfig.value) return
  try {
    providers.value = await invoke<ProviderInfo[]>('get_providers', {
      config: currentConfig.value
    })
    modelSelection.value = await invoke<ModelSelectionInfo>('get_model_selection', {
      config: currentConfig.value
    })
  } catch (error) {
    console.error('刷新提供商列表失败:', error)
  }
}

// 检查服务商是否包含主模型
const providerContainsPrimary = (providerName: string) => {
  return modelSelection.value.primary?.startsWith(`${providerName}/`) || false
}

// 设置主要模型
const setPrimaryModel = async (modelPath: string) => {
  if (!currentConfig.value) return
  try {
    currentConfig.value = await invoke<OpenClawConfig>('set_primary_model', {
      config: currentConfig.value,
      modelPath
    })
    isDirty.value = true
    await refreshProviders()
    await autoSave()
    showToast('success', `主要模型: ${modelPath}`)
  } catch (error) {
    showToast('error', `设置失败: ${error}`)
  }
}

// 设置备用模型
const setFallbackModel = async (modelPath: string) => {
  if (!currentConfig.value) return
  try {
    let newFallbacks: string[]
    if (!modelPath) {
      newFallbacks = []
    } else if (modelSelection.value.fallbacks.includes(modelPath)) {
      newFallbacks = modelSelection.value.fallbacks.filter(f => f !== modelPath)
    } else {
      newFallbacks = [...modelSelection.value.fallbacks, modelPath]
    }

    currentConfig.value = await invoke<OpenClawConfig>('set_fallback_models', {
      config: currentConfig.value,
      fallbacks: newFallbacks
    })
    isDirty.value = true
    await refreshProviders()
    await autoSave()
    showToast('success', '备用模型已更新')
  } catch (error) {
    showToast('error', `设置失败: ${error}`)
  }
}

// 打开添加提供商弹窗
const openProviderModal = () => {
  newProvider.value = { name: '', baseUrl: '', apiKey: '' }
  isEditingProvider.value = false
  editingProviderName.value = ''
  providerModalTab.value = 'manual'
  pasteJsonText.value = ''
  pasteProviderName.value = ''
  pasteApiKey.value = ''
  pasteParseError.value = ''
  parsedProviderConfig.value = null
  showProviderModal.value = true
}

// 打开编辑提供商弹窗（始终手动模式）
const openEditProviderModal = (providerName: string) => {
  const providerConfig = currentConfig.value?.models?.providers?.[providerName]
  if (!providerConfig) return
  newProvider.value = {
    name: providerName,
    baseUrl: providerConfig.baseUrl || '',
    apiKey: providerConfig.apiKey || ''
  }
  isEditingProvider.value = true
  editingProviderName.value = providerName
  providerModalTab.value = 'manual'
  showProviderModal.value = true
}

// 解析粘贴的 JSON 配置（调用纯函数）
import { parseProviderJson as _parseJson } from './utils/parseProviderJson'

const handlePasteJsonChange = (text: string) => {
  if (!text.trim()) {
    parsedProviderConfig.value = null
    pasteParseError.value = ''
    return
  }

  const result = _parseJson(text)
  parsedProviderConfig.value = result.provider
  pasteParseError.value = result.error

  if (result.provider) {
    // 同步 apiKey 到独立输入框（仅首次解析时）
    if (result.provider.apiKey && result.provider.apiKey !== 'YOUR_API_KEY') {
      pasteApiKey.value = result.provider.apiKey
    }
    // 同步名称
    if (result.name && !pasteProviderName.value) {
      pasteProviderName.value = result.name
    }
  }
}

// 监听 JSON 文本变化实时解析
watch(pasteJsonText, (val) => {
  handlePasteJsonChange(val)
})

// API Key 输入框变更时反向同步到解析结果
watch(pasteApiKey, (val) => {
  if (parsedProviderConfig.value) {
    parsedProviderConfig.value = { ...parsedProviderConfig.value, apiKey: val }
  }
})

// 添加提供商
const addProvider = async () => {
  if (!currentConfig.value) return

  // 粘贴模式
  if (providerModalTab.value === 'paste') {
    if (!pasteProviderName.value.trim()) {
      showToast('error', '请填写服务商名称')
      return
    }
    if (!parsedProviderConfig.value) {
      showToast('error', '请粘贴有效的 JSON 配置')
      return
    }
    if (!pasteApiKey.value.trim()) {
      showToast('error', '请填写 API Key')
      return
    }

    loading.value = true
    const name = pasteProviderName.value.trim()
    try {
      // 构建完整的 provider 配置，用独立输入框的 apiKey 覆盖
      const providerJson = {
        ...parsedProviderConfig.value,
        apiKey: pasteApiKey.value.trim()
      }
      currentConfig.value = await invoke<OpenClawConfig>('import_provider', {
        config: currentConfig.value,
        name,
        providerJson
      })
      isDirty.value = true
      await refreshProviders()
      await autoSave()
      showProviderModal.value = false
      const modelCount = parsedProviderConfig.value.models?.length || 0
      showToast('success', `已导入: ${name}（${modelCount} 个模型）`)
    } catch (error) {
      showToast('error', `导入失败: ${error}`)
    } finally {
      loading.value = false
    }
    return
  }

  // 手动模式（原有逻辑）
  if (!newProvider.value.name.trim() || !newProvider.value.baseUrl.trim()) {
    showToast('error', '请填写服务商名称和 Base URL')
    return
  }

  loading.value = true
  const providerNameToAdd = newProvider.value.name.trim()
  try {
    currentConfig.value = await invoke<OpenClawConfig>('upsert_provider', {
      config: currentConfig.value,
      name: providerNameToAdd,
      baseUrl: newProvider.value.baseUrl.trim(),
      apiKey: newProvider.value.apiKey.trim() || null,
      api: null
    })
    isDirty.value = true
    await refreshProviders()
    await autoSave()

    showProviderModal.value = false
    newProvider.value = { name: '', baseUrl: '', apiKey: '' }
    showToast('success', `${isEditingProvider.value ? '已更新' : '已添加'}: ${providerNameToAdd}`)
  } catch (error) {
    showToast('error', `添加失败: ${error}`)
  } finally {
    loading.value = false
  }
}

// 删除提供商
const deleteProvider = async (providerName: string) => {
  if (!currentConfig.value) return
  const confirmed = await ask(`确定要删除提供商 "${providerName}" 吗？`, { title: '确认删除', type: 'warning' })
  if (!confirmed) return

  try {
    currentConfig.value = await invoke<OpenClawConfig>('delete_provider', {
      config: currentConfig.value,
      name: providerName
    })
    isDirty.value = true
    await refreshProviders()
    await autoSave()
    showToast('success', `已删除: ${providerName}`)
  } catch (error) {
    showToast('error', `删除失败: ${error}`)
  }
}

// 打开添加模型弹窗
const openModelModal = (providerName: string) => {
  modelModalProvider.value = providerName
  newModelId.value = ''
  availableModels.value = []
  showModelDropdown.value = false
  showModelModal.value = true
}

// 添加模型到提供商
const addModelToProvider = async (providerName: string, modelId: string) => {
  if (!currentConfig.value) return
  try {
    currentConfig.value = await invoke<OpenClawConfig>('add_model_to_provider', {
      config: currentConfig.value,
      providerName,
      modelId,
      modelName: null
    })
    isDirty.value = true
    await refreshProviders()
    await autoSave()
    showToast('success', `已添加: ${providerName}/${modelId}`)
  } catch (error) {
    showToast('error', `添加失败: ${error}`)
  }
}

// 从弹窗添加模型
const addModelFromModal = async () => {
  if (!newModelId.value.trim()) {
    showToast('error', '请输入模型 ID')
    return
  }
  await addModelToProvider(modelModalProvider.value, newModelId.value.trim())
  showModelModal.value = false
  newModelId.value = ''
}

// 从下拉列表选择模型
const selectModelFromDropdown = (model: string) => {
  newModelId.value = model
  showModelDropdown.value = false
}

// 获取提供商的模型列表
const fetchModelsForProvider = async (providerName: string) => {
  const provider = providers.value.find(p => p.name === providerName)
  if (!provider) return

  // 检查是否配置了 API Key
  const providerConfig = currentConfig.value?.models?.providers?.[providerName]
  if (!providerConfig?.apiKey) {
    showToast('error', '该服务商未配置 API Key，无法获取模型列表')
    return
  }

  loadingModels.value = true
  try {
    const models = await invoke<string[]>('fetch_provider_models', {
      baseUrl: provider.baseUrl,
      apiKey: providerConfig.apiKey
    })
    availableModels.value = models
  } catch (error) {
    showToast('error', `获取模型失败: ${error}`)
  } finally {
    loadingModels.value = false
  }
}

// 下拉框打开时触发加载
const handleDropdownOpen = async () => {
  showModelDropdown.value = true
  if (availableModels.value.length === 0 && !loadingModels.value) {
    await fetchModelsForProvider(modelModalProvider.value)
  }
}

// 从提供商删除模型
const removeModelFromProvider = async (providerName: string, modelId: string) => {
  if (!currentConfig.value) return
  try {
    currentConfig.value = await invoke<OpenClawConfig>('remove_model_from_provider', {
      config: currentConfig.value,
      providerName,
      modelId
    })
    isDirty.value = true
    await refreshProviders()
    await autoSave()
    showToast('success', `已删除: ${providerName}/${modelId}`)
  } catch (error) {
    showToast('error', `删除失败: ${error}`)
  }
}

// 从备用列表移除模型
const removeFallbackModel = async (modelPath: string) => {
  if (!currentConfig.value) return
  try {
    const newFallbacks = modelSelection.value.fallbacks.filter(f => f !== modelPath)
    currentConfig.value = await invoke<OpenClawConfig>('set_fallback_models', {
      config: currentConfig.value,
      fallbacks: newFallbacks
    })
    isDirty.value = true
    await refreshProviders()
    await autoSave()
    showToast('success', `已移除: ${modelPath}`)
  } catch (error) {
    showToast('error', `移除失败: ${error}`)
  }
}

// 获取所有可用模型列表
const allAvailableModels = computed(() => {
  const models: { path: string; label: string; provider: string }[] = []
  for (const provider of providers.value) {
    for (const model of provider.models) {
      const path = `${provider.name}/${model.id}`
      models.push({
        path,
        label: model.name || model.id,
        provider: provider.name
      })
    }
  }
  return models
})

// 可添加为备用的模型（排除主模型）
const availableForFallback = computed(() => {
  const currentPrimary = modelSelection.value.primary
  const currentFallbacks = modelSelection.value.fallbacks
  return allAvailableModels.value.filter(m =>
    m.path !== currentPrimary && !currentFallbacks.includes(m.path)
  )
})

// 可选为主模型的列表（所有模型，排除当前主模型）
const availableForPrimary = computed(() => {
  const currentPrimary = modelSelection.value.primary
  return allAvailableModels.value.filter(m => m.path !== currentPrimary)
})

// 选择主模型（同时从备选列表移除）
const selectPrimaryModel = async (modelPath: string) => {
  showPrimarySelector.value = false
  if (!currentConfig.value) return

  try {
    // 设置主模型
    currentConfig.value = await invoke<OpenClawConfig>('set_primary_model', {
      config: currentConfig.value,
      modelPath
    })

    // 如果该模型在备选列表中，移除它
    if (modelSelection.value.fallbacks.includes(modelPath)) {
      const newFallbacks = modelSelection.value.fallbacks.filter(f => f !== modelPath)
      currentConfig.value = await invoke<OpenClawConfig>('set_fallback_models', {
        config: currentConfig.value,
        fallbacks: newFallbacks
      })
    }

    isDirty.value = true
    await refreshProviders()
    await autoSave()
    showToast('success', `主要模型: ${modelPath}`)
  } catch (error) {
    showToast('error', `设置失败: ${error}`)
  }
}

// 备用模型下拉控制
const showFallbackSelector = ref(false)

// 工具按钮状态
const toolLoading = ref<'restart' | 'tui' | 'minimax' | null>(null)

// Toast 通知状态
const toast = ref<{ type: 'success' | 'error'; message: string } | null>(null)
let toastTimer: ReturnType<typeof setTimeout> | null = null

const showToast = (type: 'success' | 'error', message: string) => {
  if (toastTimer) clearTimeout(toastTimer)
  toast.value = { type, message }
  toastTimer = setTimeout(() => {
    toast.value = null
  }, 4000)
}

const closeToast = () => {
  if (toastTimer) clearTimeout(toastTimer)
  toast.value = null
}
const addFallbackModel = async (modelPath: string) => {
  showFallbackSelector.value = false
  await setFallbackModel(modelPath)
}

// 点击外部关闭下拉和弹窗
const handleClickOutside = (event: MouseEvent) => {
  const target = event.target as HTMLElement
  if (!target.closest('.fallback-selector-container')) {
    showFallbackSelector.value = false
  }
  if (!target.closest('.primary-selector-container')) {
    showPrimarySelector.value = false
  }
  if (!target.closest('.model-dropdown-container')) {
    showModelDropdown.value = false
  }
}

// ============================================================================
// OpenClaw 工具函数
// ============================================================================

// 重启网关
const restartGateway = async () => {
  toolLoading.value = 'restart'
  try {
    const result = await invoke<string>('restart_gateway')
    showToast('success', result || '网关重启成功')
  } catch (error) {
    showToast('error', `重启失败: ${error}`)
  } finally {
    toolLoading.value = null
  }
}

// 打开 TUI
const openTui = async () => {
  toolLoading.value = 'tui'
  try {
    await invoke('open_tui')
    showToast('success', '已打开 TUI 终端')
  } catch (error) {
    showToast('error', `打开失败: ${error}`)
  } finally {
    toolLoading.value = null
  }
}

// Minimax 国服修复
const fixMinimaxDomestic = async () => {
  if (!currentConfig.value?.models?.providers?.['minimax']) return

  toolLoading.value = 'minimax'
  try {
    currentConfig.value.models.providers.minimax.baseUrl = 'https://api.minimaxi.com/anthropic'
    isDirty.value = true
    await autoSave()
    showToast('success', 'Minimax 国服已修复')
  } catch (error) {
    showToast('error', `修复失败: ${error}`)
  } finally {
    toolLoading.value = null
  }
}

// ============================================================================
// SSH 远程连接
// ============================================================================

// 处理指纹确认事件
const handleFingerprint = (info: FingerprintInfo, onConfirm: () => void) => {
  sshFingerprint.value = info
  sshFingerprintCallback.value = onConfirm
  showFingerprintDialog.value = true
}

// 确认指纹
const confirmFingerprint = () => {
  showFingerprintDialog.value = false
  if (sshFingerprintCallback.value) {
    sshFingerprintCallback.value()
  }
  sshFingerprint.value = null
  sshFingerprintCallback.value = null
}

// 拒绝指纹
const rejectFingerprint = async () => {
  showFingerprintDialog.value = false
  sshFingerprint.value = null
  sshFingerprintCallback.value = null
  await invoke('ssh_disconnect')
  showToast('error', '已拒绝连接')
}

// SSH 连接成功
const handleSshConnected = () => {
  sshConnected.value = true
  showSshModal.value = false
  showFileBrowser.value = true
  showToast('success', 'SSH 连接成功')
}

// SSH 断开连接
const disconnectSsh = async () => {
  try {
    await invoke('ssh_disconnect')
    sshConnected.value = false
    sshRemotePath.value = ''
    // 保留配置数据，仅重置 SSH 状态，用户可选择本地保存或重新连接
    if (isSshMode.value && fileInfo.value) {
      fileInfo.value = { ...fileInfo.value, mode: 'remote' }
    }
    showToast('success', '已断开 SSH 连接')
  } catch (e) {
    showToast('error', `断开失败: ${e}`)
  }
}

// 从远程服务器加载配置文件
const loadRemoteConfig = async (remotePath: string) => {
  showFileBrowser.value = false
  loading.value = true
  try {
    const content = await invoke<string>('ssh_read_file', { path: remotePath })
    const config: OpenClawConfig = JSON.parse(content)
    currentConfig.value = config
    sshRemotePath.value = remotePath

    const fileName = remotePath.split('/').pop() || 'openclaw.json'
    const dirPath = remotePath.substring(0, remotePath.lastIndexOf('/'))
    fileInfo.value = {
      path: remotePath,
      mode: 'ssh',
      fileName,
      dirPath,
    }
    isDirty.value = false
    await refreshProviders()
    showToast('success', `已加载远程配置: ${fileName}`)
  } catch (e) {
    showToast('error', `加载远程配置失败: ${e}`)
  } finally {
    loading.value = false
  }
}

// SSH 模式保存配置
const saveSshConfig = async () => {
  if (!currentConfig.value || !sshRemotePath.value) return
  loading.value = true
  try {
    const json = JSON.stringify(currentConfig.value, null, 2)
    await invoke('ssh_write_file', {
      path: sshRemotePath.value,
      content: json,
    })
    isDirty.value = false
    lastSaveTime.value = new Date().toLocaleString('zh-CN', { hour12: false })
    showToast('success', '已保存到远程服务器')
  } catch (e) {
    showToast('error', `远程保存失败: ${e}`)
  } finally {
    loading.value = false
  }
}

// 中国常见提供商预设
const chineseProviderPresets: ProviderPreset[] = [
  { name: 'deepseek', displayName: 'DeepSeek', baseUrl: 'https://api.deepseek.com' },
  { name: 'nvidia', displayName: '英伟达', baseUrl: 'https://integrate.api.nvidia.com/v1' },
  { name: 'siliconflow', displayName: '硅基流动', baseUrl: 'https://api.siliconflow.cn/v1' },
  { name: 'dashscope-coding', displayName: '百炼 Coding', baseUrl: 'https://coding.dashscope.aliyuncs.com/v1' },
]

const fillPreset = (preset: ProviderPreset) => {
  newProvider.value.name = preset.name
  newProvider.value.baseUrl = preset.baseUrl
}

// ============================================================================
// 生命周期
// ============================================================================

onMounted(async () => {
  await loadDefaultConfig()
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})

watch(currentConfig, () => {
  // 已在各操作中处理
}, { deep: true })
</script>

<template>
  <div class="h-screen flex flex-col bg-gray-50 dark:bg-gray-900 overflow-hidden">
    <!-- 顶部栏：标题 -->
    <header class="flex-shrink-0 border-b bg-white dark:bg-gray-800 px-4 py-2">
      <div class="flex items-center justify-between gap-4">
        <!-- 左侧：标题 -->
        <div class="flex items-center gap-2">
          <Server class="w-6 h-6 text-blue-600 flex-shrink-0" />
          <h1 class="font-bold text-lg whitespace-nowrap">OpenClawSwitch</h1>
        </div>

        <!-- 右侧：操作按钮 -->
        <div class="flex items-center gap-2 flex-shrink-0">
          <!-- SSH 连接/断开 -->
          <Button v-if="!sshConnected" variant="outline" size="sm" @click="showSshModal = true" :disabled="loading">
            <Monitor class="w-4 h-4" />
            SSH
          </Button>
          <template v-else>
            <Button variant="outline" size="sm" @click="showFileBrowser = true" :disabled="loading">
              <Monitor class="w-4 h-4 text-green-600" />
              浏览远程
            </Button>
            <Button variant="outline" size="sm" @click="disconnectSsh" :disabled="loading" class="text-red-600">
              <WifiOff class="w-4 h-4" />
            </Button>
          </template>
          <Button variant="outline" size="sm" @click="selectFile" :disabled="loading">
            <Settings class="w-4 h-4" />
            选择文件
          </Button>
          <Button variant="outline" size="sm" @click="loadLocalConfig" :disabled="loading">
            <FolderOpen class="w-4 h-4" />
            本地配置
          </Button>
          <Button v-if="isSshMode && canSave" variant="default" size="sm" @click="saveConfig" :disabled="loading || !isDirty"
                  class="bg-purple-600 hover:bg-purple-700 text-white">
            <Save class="w-4 h-4" />
            保存到远程
          </Button>
          <Button v-else-if="!isLocalMode && canSave" variant="outline" size="sm" @click="saveConfig" :disabled="loading || !isDirty">
            <Save class="w-4 h-4" />
          </Button>
          <Button v-if="canSave && !isSshMode" variant="outline" size="sm" @click="saveConfigAs" :disabled="loading">
            <Download class="w-4 h-4" />
          </Button>
          <Button v-if="currentConfig" variant="outline" size="sm" @click="showSourceModal = true">
            <FileCode class="w-4 h-4" />
            源文件
          </Button>
        </div>
      </div>

      <!-- 文件路径信息行 -->
      <div v-if="fileInfo" class="flex items-center gap-2 mt-2 text-sm">
        <span class="px-1.5 py-0.5 rounded text-xs font-medium"
              :class="isLocalMode
                ? 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300'
                : isSshMode
                  ? 'bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300'
                  : 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300'">
          {{ isLocalMode ? '本地' : isSshMode ? 'SSH' : '远程' }}
        </span>
        <span class="text-muted-foreground truncate" :title="fileInfo.path">
          {{ fileInfo.path }}
        </span>
        <span v-if="isDirty && !isLocalMode" class="text-amber-500">●</span>
        <!-- 保存时间标签 -->
        <span v-if="lastSaveTime" class="px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300">
          保存于 {{ lastSaveTime }}
        </span>
      </div>
    </header>

    <!-- 主内容区域 - 高度占满 -->
    <main class="flex-1 overflow-hidden p-4">
      <div class="max-w-6xl mx-auto h-full flex flex-col">
        <!-- 当前模型配置 + 提供商列表并排 - 占满高度 -->
        <div class="grid lg:grid-cols-3 gap-4 flex-1 min-h-0">
          <!-- 左侧：当前模型配置 -->
          <Card v-if="currentConfig" class="p-4 lg:col-span-1 overflow-auto">
            <h3 class="font-semibold mb-3 flex items-center gap-2">
              <ListTree class="w-4 h-4 text-blue-600" />
              模型配置
            </h3>

            <!-- 主要模型（下拉选择） -->
            <div class="mb-4">
              <p class="text-xs text-muted-foreground mb-1">主要模型</p>
              <div class="relative primary-selector-container">
                <Button variant="outline" size="sm" @click="showPrimarySelector = !showPrimarySelector"
                        class="w-full h-auto min-h-8 py-1.5 text-left justify-between"
                        :disabled="allAvailableModels.length === 0">
                  <span v-if="modelSelection.primary" class="text-sm text-blue-700 dark:text-blue-300 truncate">
                    {{ modelSelection.primary }}
                  </span>
                  <span v-else class="text-sm text-muted-foreground">选择主模型</span>
                  <ChevronDown class="w-4 h-4 flex-shrink-0" :class="{ 'rotate-180': showPrimarySelector }" />
                </Button>
                <div v-if="showPrimarySelector" class="absolute z-20 mt-1 w-full max-h-48 overflow-auto bg-white dark:bg-gray-800 border rounded-lg shadow-lg">
                  <div v-for="model in availableForPrimary" :key="model.path" @click="selectPrimaryModel(model.path)"
                       class="px-3 py-2 hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer text-sm border-b last:border-b-0">
                    <div class="font-medium truncate">{{ model.label }}</div>
                    <div class="text-xs text-muted-foreground truncate">{{ model.path }}</div>
                  </div>
                  <p v-if="availableForPrimary.length === 0" class="px-3 py-2 text-xs text-muted-foreground">
                    没有可选模型
                  </p>
                </div>
              </div>
            </div>

            <!-- 备用模型 -->
            <div>
              <p class="text-xs text-muted-foreground mb-2">备用模型</p>
              <div class="space-y-1">
                <div v-for="fb in modelSelection.fallbacks" :key="fb" class="flex items-center gap-1 group">
                  <code class="text-xs bg-amber-50 dark:bg-amber-950 text-amber-700 dark:text-amber-300 px-2 py-1 rounded flex-1 truncate">
                    {{ fb }}
                  </code>
                  <Button variant="ghost" size="sm" @click="removeFallbackModel(fb)"
                          class="h-6 w-6 p-0 opacity-0 group-hover:opacity-100 text-destructive">
                    <X class="w-3 h-3" />
                  </Button>
                </div>

                <!-- 添加备用模型 -->
                <div v-if="availableForFallback.length > 0" class="relative fallback-selector-container">
                  <Button variant="outline" size="sm" @click="showFallbackSelector = !showFallbackSelector" class="w-full h-7 text-xs justify-start gap-1">
                    <Plus class="w-3 h-3" />
                    添加备用
                    <ChevronDown class="w-3 h-3 ml-auto" :class="{ 'rotate-180': showFallbackSelector }" />
                  </Button>
                  <div v-if="showFallbackSelector" class="absolute z-20 mt-1 w-full max-h-48 overflow-auto bg-white dark:bg-gray-800 border rounded-lg shadow-lg">
                    <div v-for="model in availableForFallback" :key="model.path" @click="addFallbackModel(model.path)"
                         class="px-3 py-2 hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer text-sm border-b last:border-b-0">
                      <div class="font-medium truncate">{{ model.label }}</div>
                      <div class="text-xs text-muted-foreground truncate">{{ model.path }}</div>
                    </div>
                  </div>
                </div>
                <p v-else-if="modelSelection.fallbacks.length === 0" class="text-xs text-muted-foreground">
                  请先添加模型
                </p>
              </div>
            </div>

            <!-- 工具按钮区域 -->
            <div class="mt-4 pt-4 border-t">
              <p class="text-xs text-muted-foreground mb-2 flex items-center gap-1">
                <Wrench class="w-3 h-3" />
                工具
              </p>
              <div class="space-y-2">
                <Button
                  variant="outline"
                  size="sm"
                  @click="restartGateway"
                  :disabled="toolLoading !== null || isSshMode"
                  class="w-full justify-start gap-2"
                >
                  <RefreshCw class="w-4 h-4" :class="{ 'animate-spin': toolLoading === 'restart' }" />
                  重启网关
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  @click="openTui"
                  :disabled="toolLoading !== null || isSshMode"
                  class="w-full justify-start gap-2"
                >
                  <Terminal class="w-4 h-4" />
                  打开 TUI
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  @click="fixMinimaxDomestic"
                  :disabled="toolLoading !== null || !hasMinimaxProvider"
                  class="w-full justify-start gap-2"
                >
                  <Hammer class="w-4 h-4" :class="{ 'animate-pulse': toolLoading === 'minimax' }" />
                  Minimax 国服修复
                </Button>
              </div>
            </div>
          </Card>

          <!-- 右侧：提供商列表 -->
          <Card class="p-4 lg:col-span-2 flex flex-col overflow-hidden">
            <div class="flex items-center justify-between mb-3">
              <h3 class="font-semibold flex items-center gap-2">
                <Server class="w-4 h-4" />
                服务商
                <span class="px-1.5 py-0.5 bg-gray-100 dark:bg-gray-800 text-xs rounded">{{ providers.length }}</span>
              </h3>
              <Button variant="default" size="sm" @click="openProviderModal" :disabled="!currentConfig">
                <Plus class="w-4 h-4" />
                添加
              </Button>
            </div>

            <div v-if="providers.length === 0" class="text-center py-8 text-muted-foreground">
              <Server class="w-10 h-10 mx-auto mb-2 opacity-20" />
              <p class="text-sm">{{ currentConfig ? '点击添加按钮创建' : '请先加载配置文件' }}</p>
            </div>

            <div v-else class="space-y-2 flex-1 overflow-auto">
              <ProviderCard
                v-for="provider in providers"
                :key="provider.name"
                :provider="provider"
                :contains-primary="providerContainsPrimary(provider.name)"
                @set-primary="setPrimaryModel"
                @set-fallback="setFallbackModel"
                @add-model="openModelModal(provider.name)"
                @remove-model="removeModelFromProvider"
                @edit="openEditProviderModal(provider.name)"
                @delete="deleteProvider(provider.name)"
              />
            </div>
          </Card>
        </div>
      </div>
    </main>

    <!-- 添加提供商弹窗 -->
    <div v-if="showProviderModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showProviderModal = false">
      <Card class="w-full max-w-md p-6 m-4">
        <h3 class="font-semibold text-lg mb-4">{{ isEditingProvider ? '编辑服务商' : '添加服务商' }}</h3>

        <!-- 分段器（编辑模式不显示） -->
        <div v-if="!isEditingProvider" class="flex rounded-lg bg-gray-100 dark:bg-gray-800 p-1 mb-4">
          <button
            @click="providerModalTab = 'manual'"
            class="flex-1 px-3 py-1.5 text-sm rounded-md transition-colors"
            :class="providerModalTab === 'manual' ? 'bg-white dark:bg-gray-700 shadow-sm font-medium' : 'text-muted-foreground hover:text-foreground'"
          >
            手动配置
          </button>
          <button
            @click="providerModalTab = 'paste'"
            class="flex-1 px-3 py-1.5 text-sm rounded-md transition-colors"
            :class="providerModalTab === 'paste' ? 'bg-white dark:bg-gray-700 shadow-sm font-medium' : 'text-muted-foreground hover:text-foreground'"
          >
            粘贴配置
          </button>
        </div>

        <!-- 手动配置模式 -->
        <div v-if="providerModalTab === 'manual' || isEditingProvider" class="space-y-4">
          <div>
            <Label class="text-sm mb-1 block">服务商名称 *</Label>
            <Input v-model="newProvider.name" placeholder="例如: openai" :disabled="loading || isEditingProvider" />
          </div>
          <div>
            <Label class="text-sm mb-1 block">Base URL *</Label>
            <Input v-model="newProvider.baseUrl" placeholder="https://api.openai.com/v1" :disabled="loading" />
          </div>
          <div>
            <Label class="text-sm mb-1 block">API Key <span class="text-muted-foreground">(可选)</span></Label>
            <Input v-model="newProvider.apiKey" type="password" placeholder="sk-..." :disabled="loading" />
          </div>

          <!-- 快速选择标签 -->
          <div class="flex flex-wrap items-center gap-2">
            <span class="text-xs text-muted-foreground">快速选择:</span>
            <button
              v-for="preset in chineseProviderPresets"
              :key="preset.name"
              @click="fillPreset(preset)"
              class="px-2 py-1 text-xs rounded-md bg-blue-50 text-blue-700 hover:bg-blue-100 dark:bg-blue-900 dark:text-blue-200 transition-colors"
            >
              {{ preset.displayName }}
            </button>
          </div>
        </div>

        <!-- 粘贴配置模式 -->
        <div v-if="providerModalTab === 'paste' && !isEditingProvider" class="space-y-3">
          <div>
            <Label class="text-sm mb-1 block">服务商名称 *</Label>
            <Input v-model="pasteProviderName" placeholder="例如: bailian" :disabled="loading" />
          </div>
          <div>
            <Label class="text-sm mb-1 block">JSON 配置 *</Label>
            <textarea
              v-model="pasteJsonText"
              placeholder='粘贴服务商提供的 JSON 配置，支持包含 providers 的完整配置或单个服务商配置'
              :disabled="loading"
              rows="8"
              class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 resize-none font-mono"
            />
            <!-- 解析状态 -->
            <p v-if="pasteParseError" class="text-xs text-red-500 mt-1">{{ pasteParseError }}</p>
            <p v-else-if="parsedProviderConfig" class="text-xs text-green-600 dark:text-green-400 mt-1">
              ✓ 解析成功，包含 {{ parsedProviderConfig.models?.length || 0 }} 个模型
            </p>
          </div>
          <div>
            <Label class="text-sm mb-1 block">API Key *</Label>
            <Input v-model="pasteApiKey" type="password" placeholder="sk-..." :disabled="loading" />
          </div>
        </div>

        <div class="flex justify-end gap-2 mt-6">
          <Button variant="ghost" @click="showProviderModal = false">取消</Button>
          <Button @click="addProvider" :disabled="loading">
            <Plus v-if="!isEditingProvider" class="w-4 h-4" />
            {{ isEditingProvider ? '保存' : providerModalTab === 'paste' ? '导入' : '添加' }}
          </Button>
        </div>
      </Card>
    </div>

    <!-- 添加模型弹窗 -->
    <div v-if="showModelModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showModelModal = false">
      <Card class="w-full max-w-sm p-6 m-4">
        <h3 class="font-semibold text-lg mb-4">添加模型到 {{ modelModalProvider }}</h3>

        <div class="space-y-4">
          <div>
            <Label class="text-sm mb-1 block">模型 ID *</Label>
            <div class="relative model-dropdown-container">
              <Input
                :value="newModelId"
                @input="newModelId = ($event.target as HTMLInputElement).value"
                @focus="handleDropdownOpen"
                placeholder="搜索模型或手动输入"
                @keyup.enter="addModelFromModal"
                autocomplete="off"
                autocorrect="off"
                autocapitalize="off"
                spellcheck="false"
                lang="en"
              />
              <!-- 下拉列表 -->
              <div
                v-if="showModelDropdown && (availableModels.length > 0 || loadingModels)"
                class="absolute inset-x-0 top-full mt-1 max-h-48 overflow-auto bg-white dark:bg-gray-800 border rounded-lg shadow-lg z-10"
                @click.stop
              >
                <div v-if="loadingModels" class="px-3 py-2 text-xs text-muted-foreground">
                  加载中...
                </div>
                <template v-else>
                  <div
                    v-for="model in filteredModels"
                    :key="model"
                    @click="selectModelFromDropdown(model)"
                    class="px-3 py-2 hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer text-sm border-b last:border-b-0"
                  >
                    {{ model }}
                  </div>
                  <p v-if="filteredModels.length === 0" class="px-3 py-2 text-xs text-muted-foreground">
                    无匹配结果
                  </p>
                </template>
              </div>
            </div>
          </div>
        </div>

        <div class="flex justify-end gap-2 mt-6">
          <Button variant="ghost" @click="showModelModal = false">取消</Button>
          <Button @click="addModelFromModal" :disabled="!newModelId.trim()">
            <Plus class="w-4 h-4" />
            添加
          </Button>
        </div>
      </Card>
    </div>

    <!-- 源文件查看弹窗 -->
    <div v-if="showSourceModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showSourceModal = false">
      <Card class="w-full max-w-4xl max-h-[85vh] m-4 flex flex-col">
        <div class="flex items-center justify-between p-4 border-b">
          <h3 class="font-semibold text-lg flex items-center gap-2">
            <FileCode class="w-5 h-5" />
            源文件内容
          </h3>
          <div class="flex items-center gap-2">
            <span v-if="fileInfo" class="text-xs text-muted-foreground truncate max-w-md">
              {{ fileInfo.path }}
            </span>
            <Button variant="ghost" size="sm" @click="showSourceModal = false" class="h-8 w-8 p-0">
              <X class="w-4 h-4" />
            </Button>
          </div>
        </div>
        <div class="flex-1 overflow-auto p-4">
          <pre class="text-xs bg-gray-50 dark:bg-gray-900 p-4 rounded-lg overflow-x-auto">{{ JSON.stringify(currentConfig, null, 2) }}</pre>
        </div>
      </Card>
    </div>

    <!-- SSH 连接弹窗 -->
    <SshConnectModal
      v-if="showSshModal"
      @close="showSshModal = false"
      @connected="handleSshConnected"
      @fingerprint="handleFingerprint"
    />

    <!-- SSH 指纹确认弹窗 -->
    <SshFingerprintDialog
      v-if="showFingerprintDialog && sshFingerprint"
      :fingerprint="sshFingerprint"
      @confirm="confirmFingerprint"
      @reject="rejectFingerprint"
    />

    <!-- 远程文件浏览器 -->
    <RemoteFileBrowser
      v-if="showFileBrowser"
      @close="showFileBrowser = false"
      @select="loadRemoteConfig"
    />

    <!-- Toast 通知 -->
    <Toast
      v-if="toast"
      :type="toast.type"
      :message="toast.message"
      @close="closeToast"
    />
  </div>
</template>
