<script setup lang="ts">
import { ref, computed, onMounted, watch, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { open, save } from '@tauri-apps/api/dialog'
import Button from './components/ui/Button.vue'
import Input from './components/ui/Input.vue'
import Label from './components/ui/Label.vue'
import Card from './components/ui/Card.vue'
import Alert from './components/ui/Alert.vue'
import Toast from './components/ui/Toast.vue'
import FileSelector from './components/FileSelector.vue'
import ProviderCard from './components/ProviderCard.vue'
import {
  Server, Key, Globe, Settings, CheckCircle2, XCircle,
  Eye, ListTree, Save, Download, Plus, X, ChevronDown, FolderOpen, FileCode,
  RefreshCw, Terminal, Wrench
} from 'lucide-vue-next'
import type {
  OpenClawConfig, ProviderInfo, ModelSelectionInfo, ConfigFileInfo
} from './types/config'

// ============================================================================
// 状态管理
// ============================================================================

// 配置状态
const currentConfig = ref<OpenClawConfig | null>(null)
const fileInfo = ref<ConfigFileInfo | null>(null)
const isDirty = ref(false)

// 提供商和模型状态
const providers = ref<ProviderInfo[]>([])
const modelSelection = ref<ModelSelectionInfo>({ primary: null, fallbacks: [] })

// UI 状态
const loading = ref(false)
const message = ref('')
const messageType = ref<'success' | 'error' | ''>('')
const showConfig = ref(false)

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

// 新模型表单
const newModelId = ref('')

// ============================================================================
// 计算属性
// ============================================================================

const isLocalMode = computed(() => fileInfo.value?.mode === 'local')
const canSave = computed(() => currentConfig.value && fileInfo.value)

// ============================================================================
// 文件操作
// ============================================================================

// 加载默认配置
const loadDefaultConfig = async () => {
  loading.value = true
  message.value = ''
  try {
    const [config, info] = await invoke<[OpenClawConfig, ConfigFileInfo]>('load_default_config')
    currentConfig.value = config
    fileInfo.value = info
    isDirty.value = false
    await refreshProviders()
    message.value = `已加载: ${info.fileName}`
    messageType.value = 'success'
  } catch (error) {
    console.error('加载默认配置失败:', error)
    message.value = `${error}`
    messageType.value = 'error'
  } finally {
    loading.value = false
  }
}

// 选择配置目录
const selectDirectory = async () => {
  try {
    const selected = await open({
      directory: true,
      title: '选择 OpenClaw 配置目录'
    })
    if (selected && typeof selected === 'string') {
      loading.value = true
      const [config, info] = await invoke<[OpenClawConfig, ConfigFileInfo]>(
        'load_config_from_directory',
        { dirPath: selected }
      )
      currentConfig.value = config
      fileInfo.value = info
      isDirty.value = false
      await refreshProviders()
      message.value = `已加载: ${info.fileName}`
      messageType.value = 'success'
    }
  } catch (error) {
    console.error('选择目录失败:', error)
    message.value = `${error}`
    messageType.value = 'error'
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
      message.value = `已加载: ${info.fileName}`
      messageType.value = 'success'
    }
  } catch (error) {
    console.error('选择文件失败:', error)
    message.value = `${error}`
    messageType.value = 'error'
  } finally {
    loading.value = false
  }
}

// 重新加载
const reload = async () => {
  if (!fileInfo.value) return
  loading.value = true
  try {
    if (fileInfo.value.mode === 'local') {
      const [config, info] = await invoke<[OpenClawConfig, ConfigFileInfo]>(
        'load_config_from_directory',
        { dirPath: fileInfo.value.dirPath }
      )
      currentConfig.value = config
      fileInfo.value = info
    } else {
      const [config, info] = await invoke<[OpenClawConfig, ConfigFileInfo]>(
        'load_config_from_file',
        { filePath: fileInfo.value.path }
      )
      currentConfig.value = config
      fileInfo.value = info
    }
    isDirty.value = false
    await refreshProviders()
    message.value = '已重新加载'
    messageType.value = 'success'
  } catch (error) {
    message.value = `重新加载失败: ${error}`
    messageType.value = 'error'
  } finally {
    loading.value = false
  }
}

// 保存配置
const saveConfig = async () => {
  if (!currentConfig.value || !fileInfo.value) return
  loading.value = true
  try {
    await invoke('save_config', {
      config: currentConfig.value,
      path: fileInfo.value.path
    })
    isDirty.value = false
    message.value = '已保存'
    messageType.value = 'success'
  } catch (error) {
    message.value = `保存失败: ${error}`
    messageType.value = 'error'
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
        dirPath: selected.substring(0, selected.lastIndexOf(/[/\\]/))
      }
      isDirty.value = false
      message.value = `已保存到: ${selected}`
      messageType.value = 'success'
    }
  } catch (error) {
    message.value = `另存为失败: ${error}`
    messageType.value = 'error'
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

// 检查提供商是否为主要模型
const isPrimaryProvider = (providerName: string) => {
  return modelSelection.value.primary?.startsWith(`${providerName}/`) || false
}

// 检查提供商是否为备用模型
const isFallbackProvider = (providerName: string) => {
  return modelSelection.value.fallbacks.some(f => f.startsWith(`${providerName}/`))
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
    message.value = `主要模型: ${modelPath}`
    messageType.value = 'success'
  } catch (error) {
    message.value = `设置失败: ${error}`
    messageType.value = 'error'
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
    message.value = '备用模型已更新'
    messageType.value = 'success'
  } catch (error) {
    message.value = `设置失败: ${error}`
    messageType.value = 'error'
  }
}

// 打开添加提供商弹窗
const openProviderModal = () => {
  newProvider.value = { name: '', baseUrl: '', apiKey: '' }
  showProviderModal.value = true
}

// 添加提供商
const addProvider = async () => {
  if (!currentConfig.value) return
  if (!newProvider.value.name.trim() || !newProvider.value.baseUrl.trim()) {
    message.value = '请填写服务商名称和 Base URL'
    messageType.value = 'error'
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
    message.value = `已添加: ${providerNameToAdd}`
    messageType.value = 'success'
  } catch (error) {
    message.value = `添加失败: ${error}`
    messageType.value = 'error'
  } finally {
    loading.value = false
  }
}

// 删除提供商
const deleteProvider = async (providerName: string) => {
  if (!currentConfig.value) return
  if (!confirm(`确定要删除提供商 "${providerName}" 吗？`)) return

  try {
    currentConfig.value = await invoke<OpenClawConfig>('delete_provider', {
      config: currentConfig.value,
      name: providerName
    })
    isDirty.value = true
    await refreshProviders()
    await autoSave()
    message.value = `已删除: ${providerName}`
    messageType.value = 'success'
  } catch (error) {
    message.value = `删除失败: ${error}`
    messageType.value = 'error'
  }
}

// 打开添加模型弹窗
const openModelModal = (providerName: string) => {
  modelModalProvider.value = providerName
  newModelId.value = ''
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
    message.value = `已添加: ${providerName}/${modelId}`
    messageType.value = 'success'
  } catch (error) {
    message.value = `添加失败: ${error}`
    messageType.value = 'error'
  }
}

// 从弹窗添加模型
const addModelFromModal = async () => {
  if (!newModelId.value.trim()) {
    message.value = '请输入模型 ID'
    messageType.value = 'error'
    return
  }
  await addModelToProvider(modelModalProvider.value, newModelId.value.trim())
  showModelModal.value = false
  newModelId.value = ''
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
    message.value = `已删除: ${providerName}/${modelId}`
    messageType.value = 'success'
  } catch (error) {
    message.value = `删除失败: ${error}`
    messageType.value = 'error'
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
    message.value = `已移除: ${modelPath}`
    messageType.value = 'success'
  } catch (error) {
    message.value = `移除失败: ${error}`
    messageType.value = 'error'
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
    message.value = `主要模型: ${modelPath}`
    messageType.value = 'success'
  } catch (error) {
    message.value = `设置失败: ${error}`
    messageType.value = 'error'
  }
}

// 备用模型下拉控制
const showFallbackSelector = ref(false)

// 工具按钮状态
const toolLoading = ref<'restart' | 'tui' | null>(null)

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

// 快速示例
const examples = [
  { name: 'OpenAI', provider: 'openai', url: 'https://api.openai.com/v1' },
  { name: 'Anthropic', provider: 'anthropic', url: 'https://api.anthropic.com/v1' },
  { name: 'Ollama', provider: 'ollama', url: 'http://localhost:11434/v1' },
]

const fillExample = (example: typeof examples[0]) => {
  newProvider.value.name = example.provider
  newProvider.value.baseUrl = example.url
}

// 清除消息
const clearMessage = () => {
  message.value = ''
  messageType.value = ''
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
    <!-- 顶部栏：标题 + 文件选择 + 消息 -->
    <header class="flex-shrink-0 border-b bg-white dark:bg-gray-800 px-4 py-2">
      <div class="flex items-center justify-between gap-4">
        <!-- 左侧：标题和文件信息 -->
        <div class="flex items-center gap-4 min-w-0">
          <div class="flex items-center gap-2">
            <Server class="w-6 h-6 text-blue-600 flex-shrink-0" />
            <h1 class="font-bold text-lg whitespace-nowrap">OpenClaw Manager</h1>
          </div>

          <!-- 文件路径显示 -->
          <div v-if="fileInfo" class="flex items-center gap-2 text-sm min-w-0">
            <span class="px-1.5 py-0.5 rounded text-xs font-medium"
                  :class="isLocalMode ? 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300' : 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300'">
              {{ isLocalMode ? '本地' : '远程' }}
            </span>
            <span class="text-muted-foreground truncate max-w-xs" :title="fileInfo.path">
              {{ fileInfo.path }}
            </span>
            <span v-if="isDirty && !isLocalMode" class="text-amber-500">●</span>
          </div>
        </div>

        <!-- 右侧：操作按钮和消息 -->
        <div class="flex items-center gap-2 flex-shrink-0">
          <!-- 消息提示 -->
          <div v-if="message"
               class="flex items-center gap-1 px-2 py-1 rounded text-xs cursor-pointer"
               :class="messageType === 'success' ? 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300' : 'bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300'"
               @click="clearMessage">
            <component :is="messageType === 'success' ? CheckCircle2 : XCircle" class="w-3 h-3" />
            <span class="max-w-40 truncate">{{ message }}</span>
          </div>

          <Button variant="outline" size="sm" @click="selectDirectory" :disabled="loading">
            <FolderOpen class="w-4 h-4" />
            目录
          </Button>
          <Button variant="outline" size="sm" @click="selectFile" :disabled="loading">
            <Settings class="w-4 h-4" />
            文件
          </Button>
          <Button v-if="!isLocalMode && canSave" variant="outline" size="sm" @click="saveConfig" :disabled="loading || !isDirty">
            <Save class="w-4 h-4" />
          </Button>
          <Button v-if="canSave" variant="outline" size="sm" @click="saveConfigAs" :disabled="loading">
            <Download class="w-4 h-4" />
          </Button>
          <Button v-if="currentConfig" variant="outline" size="sm" @click="showSourceModal = true">
            <FileCode class="w-4 h-4" />
            源文件
          </Button>
        </div>
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
                  :disabled="toolLoading !== null"
                  class="w-full justify-start gap-2"
                >
                  <RefreshCw class="w-4 h-4" :class="{ 'animate-spin': toolLoading === 'restart' }" />
                  重启网关
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  @click="openTui"
                  :disabled="toolLoading !== null"
                  class="w-full justify-start gap-2"
                >
                  <Terminal class="w-4 h-4" />
                  打开 TUI
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
                :is-primary="isPrimaryProvider(provider.name)"
                :is-fallback="isFallbackProvider(provider.name)"
                @set-primary="setPrimaryModel"
                @set-fallback="setFallbackModel"
                @add-model="openModelModal(provider.name)"
                @remove-model="removeModelFromProvider"
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
        <h3 class="font-semibold text-lg mb-4">添加服务商</h3>

        <div class="space-y-4">
          <div>
            <Label class="text-sm mb-1 block">服务商名称 *</Label>
            <Input v-model="newProvider.name" placeholder="例如: openai" :disabled="loading" />
          </div>
          <div>
            <Label class="text-sm mb-1 block">Base URL *</Label>
            <Input v-model="newProvider.baseUrl" placeholder="https://api.openai.com/v1" :disabled="loading" />
          </div>
          <div>
            <Label class="text-sm mb-1 block">API Key <span class="text-muted-foreground">(可选)</span></Label>
            <Input v-model="newProvider.apiKey" type="password" placeholder="sk-..." :disabled="loading" />
          </div>

          <!-- 快速填充 -->
          <div class="flex items-center gap-2">
            <span class="text-xs text-muted-foreground">快速:</span>
            <Button v-for="ex in examples" :key="ex.provider" variant="outline" size="sm" @click="fillExample(ex)" class="h-6 text-xs">
              {{ ex.name }}
            </Button>
          </div>
        </div>

        <div class="flex justify-end gap-2 mt-6">
          <Button variant="ghost" @click="showProviderModal = false">取消</Button>
          <Button @click="addProvider" :disabled="loading">
            <Plus class="w-4 h-4" />
            添加
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
            <Input v-model="newModelId" placeholder="例如: gpt-4o, claude-3-opus" @keyup.enter="addModelFromModal" />
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

    <!-- Toast 通知 -->
    <Toast
      v-if="toast"
      :type="toast.type"
      :message="toast.message"
      @close="closeToast"
    />
  </div>
</template>
