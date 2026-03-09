<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import {
  Bot,
  CheckCircle2,
  ChevronLeft,
  Loader2,
  MessageSquareMore,
  Rocket,
  ShieldCheck,
  Sparkles,
  X,
} from 'lucide-vue-next'
import Button from '../ui/Button.vue'
import Input from '../ui/Input.vue'
import { isPrimaryModelPlaceholder } from '../../domain/configValidation'
import { DEFAULT_GATEWAY_READY_OPTIONS, waitForGatewayReady } from '../../domain/gatewayStartup'
import { formatGatewayInstallError, isAdminRequiredGatewayInstallError } from '../../domain/postInstallError'
import {
  applyQuickSetupGatewayOptions,
  applyQuickSetupModelPreset,
  buildQuickSetupModelOptions,
  clearQuickSetupManagedChannels,
  createQuickSetupCustomProviderPreset,
  filterQuickSetupModelOptions,
  QUICK_SETUP_CHANNEL_PRESETS,
  QUICK_SETUP_PROVIDER_PRESETS,
  QUICK_SETUP_STEPS,
  canSkipQuickSetupStep,
  findProviderPreset,
  sanitizeQuickSetupChannelConfig,
  type QuickSetupModelOption,
  type QuickSetupChannelId,
  type QuickSetupProviderId,
  type QuickSetupStepId,
} from '../../domain/quickSetupGuide'
import {
  clearQuickSetupSession,
  createQuickSetupSessionSnapshot,
  loadQuickSetupSession,
  resolveQuickSetupSessionStepIndex,
  saveQuickSetupSession,
  shouldPersistQuickSetupSession,
  type QuickSetupSessionStatus,
} from '../../domain/quickSetupSession'
import { mergeFeishuChannelConfig } from '../../domain/feishuPlugin'
import { resolveDingtalkChannelNode } from '../../domain/dingtalkPlugin'
import { getChannelConfigKey, isChannelPluginInstalled, type ChannelPluginStatus } from '../../domain/channelPluginCatalog'
import type { ConfigFileInfo, ModelSelectionInfo, OpenClawConfig, ProviderInfo } from '../../types/config'

const props = defineProps<{
  showToast: (type: 'success' | 'error', message: string) => void
  showCloseAction?: boolean
  systemOs: 'windows' | 'macos' | 'linux'
}>()

const emit = defineEmits<{
  complete: []
  close: []
}>()

const stepIndex = ref(0)
const busy = ref(false)
const busyMessage = ref('')
const errorMessage = ref('')
const infoMessage = ref('')
const adminRelaunching = ref(false)
const adminRelaunchMessage = ref('')

const currentConfig = ref<OpenClawConfig | null>(null)
const fileInfo = ref<ConfigFileInfo | null>(null)
const providers = ref<ProviderInfo[]>([])
const modelSelection = ref<ModelSelectionInfo>({ primary: null, fallbacks: [] })
const channelExtensionStatus = ref<ChannelPluginStatus>({
  feishuInstalled: false,
  wecomInstalled: false,
  qqInstalled: false,
  dingtalkInstalled: false,
})

const selectedProviderId = ref<QuickSetupProviderId>('dashscope-coding')
const providerApiKey = ref('')
const modelQuery = ref('')
const modelSelectionMode = ref<'auto' | 'manual'>('auto')
const customProviderName = ref('')
const customProviderBaseUrl = ref('')
const fetchedModels = ref<string[]>([])
const loadingModels = ref(false)
const modelOptionsRequestKey = ref('')
const showModelDropdown = ref(false)

const selectedChannelId = ref<QuickSetupChannelId>('feishu')
const channelIdValue = ref('')
const channelSecretValue = ref('')
const browserDefaultProfileEnabled = ref(false)
const toolsFullProfileEnabled = ref(false)

const savedStepIds = ref<QuickSetupStepId[]>([])
const restoringSession = ref(false)
const sessionPersistenceDisabled = ref(false)

const currentStep = computed(() => QUICK_SETUP_STEPS[stepIndex.value])
const canRelaunchAsAdmin = computed(() => isAdminRequiredGatewayInstallError(errorMessage.value))
const currentProviderPreset = computed(() => findProviderPreset(selectedProviderId.value)!)
const isCustomProvider = computed(() => currentProviderPreset.value.isCustom === true)
const resolvedProviderBaseUrl = computed(() =>
  isCustomProvider.value ? customProviderBaseUrl.value.trim() : currentProviderPreset.value.baseUrl
)
const resolvedProviderDisplayName = computed(() =>
  isCustomProvider.value
    ? (customProviderName.value.trim() || currentProviderPreset.value.displayName)
    : currentProviderPreset.value.displayName
)
const currentChannelPreset = computed(
  () => QUICK_SETUP_CHANNEL_PRESETS.find((item) => item.id === selectedChannelId.value) ?? QUICK_SETUP_CHANNEL_PRESETS[0]
)
const hasReadyPrimaryModel = computed(() => {
  const primary = modelSelection.value.primary
  return Boolean(primary && !isPrimaryModelPlaceholder(primary))
})
const extensionInstalled = computed(() => {
  return isChannelPluginInstalled(channelExtensionStatus.value, selectedChannelId.value)
})
const modelOptions = computed<QuickSetupModelOption[]>(() =>
  buildQuickSetupModelOptions({
    presetModels: currentProviderPreset.value.providerModels,
    fetchedModels: fetchedModels.value,
    modelQuery: modelQuery.value,
  })
)
const filteredModelOptions = computed<QuickSetupModelOption[]>(() =>
  filterQuickSetupModelOptions(modelOptions.value, modelQuery.value)
)
const hasModelOptions = computed(() => modelOptions.value.length > 0)
const shouldShowModelDropdown = computed(() =>
  showModelDropdown.value && (loadingModels.value || hasModelOptions.value || modelQuery.value.trim().length > 0)
)
const modelInputPlaceholder = computed(() => {
  const suggestedId = currentProviderPreset.value.suggestedModels[0]?.id
  return suggestedId ? `输入模型 ID，例如 ${suggestedId}` : '输入模型 ID，例如 gpt-4.1'
})
const currentModelOptionsRequestKey = computed(() => {
  if (currentProviderPreset.value.skipModelFetch) return ''
  const apiKey = providerApiKey.value.trim()
  const baseUrl = resolvedProviderBaseUrl.value.trim()
  if (!apiKey || !baseUrl) return ''
  return `${selectedProviderId.value}::${baseUrl}::${apiKey}`
})
const modelOptionBadgeLabel = computed(() => {
  if (loadingModels.value) return '正在读取列表'
  if (currentProviderPreset.value.skipModelFetch) {
    return currentProviderPreset.value.providerModels.length > 0
      ? `${currentProviderPreset.value.providerModels.length} 个预设`
      : '预设模型'
  }
  return hasModelOptions.value ? `${modelOptions.value.length} 个候选` : '自动读取'
})
const primaryButtonLabel = computed(() => {
  if (busy.value) return busyMessage.value || '处理中...'
  if (currentStep.value.id === 'model') return '保存模型并下一步'
  if (currentStep.value.id === 'channel') return '保存渠道并下一步'
  return '安装网关并进入工作台'
})
const primaryModelPath = computed(() => modelSelection.value.primary || '')
const maskSecret = (value: string) => {
  const trimmed = value.trim()
  if (!trimmed) return ''
  if (trimmed.length <= 8) return trimmed
  return trimmed.slice(0, 4) + '...' + trimmed.slice(-4)
}
const parseQqCredentials = (node: Record<string, any>) => {
  const appId = typeof node.appId === 'string' ? node.appId.trim() : ''
  const clientSecret = typeof node.clientSecret === 'string' ? node.clientSecret.trim() : ''
  if (appId || clientSecret) {
    return { appId, clientSecret }
  }

  const token = typeof node.token === 'string' ? node.token.trim() : ''
  if (!token) {
    return { appId: '', clientSecret: '' }
  }

  const [legacyAppId, ...secretParts] = token.split(':')
  return {
    appId: legacyAppId?.trim() || '',
    clientSecret: secretParts.join(':').trim(),
  }
}
const selectedChannelSummary = computed(() => {
  return channelIdValue.value.trim() || maskSecret(channelSecretValue.value)
})
const browserProfileStatusText = computed(() =>
  browserDefaultProfileEnabled.value ? '已启用浏览器默认配置' : '未启用'
)
const toolsProfileStatusText = computed(() =>
  toolsFullProfileEnabled.value ? '已启用完整工具能力' : '未启用'
)

const asRecord = (value: unknown): Record<string, any> =>
  value && typeof value === 'object' && !Array.isArray(value) ? (value as Record<string, any>) : {}

const markStepSaved = (stepId: QuickSetupStepId) => {
  if (!savedStepIds.value.includes(stepId)) {
    savedStepIds.value = [...savedStepIds.value, stepId]
  }
}

const persistQuickSetupSession = (status: QuickSetupSessionStatus = 'in_progress') => {
  if (!shouldPersistQuickSetupSession({
    restoringSession: restoringSession.value,
    persistenceDisabled: sessionPersistenceDisabled.value,
  })) return
  const currentStepId = QUICK_SETUP_STEPS[stepIndex.value]?.id ?? 'model'
  saveQuickSetupSession(createQuickSetupSessionSnapshot({
    status,
    stepId: currentStepId,
    savedStepIds: savedStepIds.value,
    selectedProviderId: selectedProviderId.value,
    providerApiKey: providerApiKey.value,
    modelQuery: modelQuery.value,
    modelSelectionMode: modelSelectionMode.value,
    customProviderName: customProviderName.value,
    customProviderBaseUrl: customProviderBaseUrl.value,
    selectedChannelId: selectedChannelId.value,
    channelIdValue: channelIdValue.value,
    channelSecretValue: channelSecretValue.value,
    browserDefaultProfileEnabled: browserDefaultProfileEnabled.value,
    toolsFullProfileEnabled: toolsFullProfileEnabled.value,
  }))
}

const stopQuickSetupSessionPersistence = () => {
  sessionPersistenceDisabled.value = true
  clearQuickSetupSession()
}

const restoreQuickSetupSessionState = () => {
  const snapshot = loadQuickSetupSession()
  if (!snapshot) return

  restoringSession.value = true
  try {
    selectedProviderId.value = snapshot.selectedProviderId
    selectedChannelId.value = snapshot.selectedChannelId
    providerApiKey.value = snapshot.providerApiKey
    modelQuery.value = snapshot.modelQuery
    modelSelectionMode.value = snapshot.modelSelectionMode
    customProviderName.value = snapshot.customProviderName
    customProviderBaseUrl.value = snapshot.customProviderBaseUrl
    channelIdValue.value = snapshot.channelIdValue
    channelSecretValue.value = snapshot.channelSecretValue
    browserDefaultProfileEnabled.value = snapshot.browserDefaultProfileEnabled
    toolsFullProfileEnabled.value = snapshot.toolsFullProfileEnabled
    savedStepIds.value = [...snapshot.savedStepIds]
    stepIndex.value = resolveQuickSetupSessionStepIndex(snapshot)

    if (snapshot.status === 'awaiting_admin_relaunch') {
      errorMessage.value = ''
      adminRelaunchMessage.value = ''
      infoMessage.value = '已恢复管理员重启前的快速引导进度，请继续完成网关安装。'
    }
  } finally {
    restoringSession.value = false
  }
}

const setBusy = (message: string) => {
  busy.value = true
  busyMessage.value = message
  errorMessage.value = ''
  adminRelaunchMessage.value = ''
}

const clearBusy = () => {
  busy.value = false
  busyMessage.value = ''
}

const selectModelFromDropdown = (modelId: string) => {
  modelQuery.value = modelId
  showModelDropdown.value = false
}

const moveNext = () => {
  if (stepIndex.value < QUICK_SETUP_STEPS.length - 1) {
    stepIndex.value += 1
  }
}

const movePrev = () => {
  if (stepIndex.value > 0) {
    stepIndex.value -= 1
  }
}

const skipCurrentStep = () => {
  if (!canSkipQuickSetupStep(currentStep.value.id)) return
  infoMessage.value = `已跳过“${currentStep.value.title}”，稍后仍可在对应页面继续配置。`
  errorMessage.value = ''
  moveNext()
}

const saveConfigToDisk = async () => {
  if (!currentConfig.value || !fileInfo.value) {
    throw new Error('配置文件未就绪')
  }
  await invoke('save_config', {
    config: currentConfig.value,
    path: fileInfo.value.path,
  })
}

const refreshProvidersAndSelection = async () => {
  if (!currentConfig.value) return
  providers.value = await invoke<ProviderInfo[]>('get_providers', { config: currentConfig.value })
  modelSelection.value = await invoke<ModelSelectionInfo>('get_model_selection', { config: currentConfig.value })
}

const refreshChannelExtensionStatus = async () => {
  channelExtensionStatus.value = await invoke<ChannelPluginStatus>('get_channel_extension_status')
}

const hydrateDraftsFromConfig = () => {
  if (!currentConfig.value) return

  const primary = modelSelection.value.primary || ''
  if (primary) {
    const [providerName, modelId] = primary.split('/', 2)
    if (modelId) modelQuery.value = modelId
    const providerConfig = currentConfig.value.models?.providers?.[providerName]
    if (providerConfig?.apiKey) {
      providerApiKey.value = providerConfig.apiKey
    }
    const preset = ['dashscope-coding', 'tencent-coding', 'deepseek', 'dashscope', 'siliconflow']
      .map((id) => findProviderPreset(id))
      .find((item) => item && (item.name === providerName || item.baseUrl === providerConfig?.baseUrl))
    if (preset) {
      selectedProviderId.value = preset.id
      modelQuery.value = modelId || ''
      modelSelectionMode.value = 'auto'
      customProviderName.value = ''
      customProviderBaseUrl.value = ''
    } else {
      selectedProviderId.value = 'custom'
      modelSelectionMode.value = 'manual'
      customProviderName.value = providerName
      customProviderBaseUrl.value = providerConfig?.baseUrl || ''
    }
    markStepSaved('model')
  }

  const channels = asRecord(currentConfig.value.channels)
  const feishu = asRecord(channels.feishu)
  const wecom = asRecord(channels.wecom)
  const qq = asRecord(channels.qqbot)
  const dingtalk = asRecord(resolveDingtalkChannelNode(channels))
  const browser = asRecord(currentConfig.value.browser)
  const tools = asRecord(currentConfig.value.tools)

  browserDefaultProfileEnabled.value = browser.defaultProfile === 'openclaw'
  toolsFullProfileEnabled.value = tools.profile === 'full'

  if (feishu.appId && feishu.appSecret) {
    selectedChannelId.value = 'feishu'
    channelIdValue.value = String(feishu.appId)
    channelSecretValue.value = String(feishu.appSecret)
    markStepSaved('channel')
    return
  }
  if (wecom.botId && wecom.secret) {
    selectedChannelId.value = 'wecom'
    channelIdValue.value = String(wecom.botId)
    channelSecretValue.value = String(wecom.secret)
    markStepSaved('channel')
    return
  }
  const qqCredentials = parseQqCredentials(qq)
  if (qqCredentials.appId && qqCredentials.clientSecret) {
    selectedChannelId.value = 'qq'
    channelIdValue.value = qqCredentials.appId
    channelSecretValue.value = qqCredentials.clientSecret
    markStepSaved('channel')
    return
  }
  if (dingtalk.clientId && dingtalk.clientSecret) {
    selectedChannelId.value = 'dingtalk'
    channelIdValue.value = String(dingtalk.clientId)
    channelSecretValue.value = String(dingtalk.clientSecret)
    markStepSaved('channel')
  }
}

const loadConfig = async () => {
  const [config, info] = await invoke<[OpenClawConfig, ConfigFileInfo]>('load_default_config')
  currentConfig.value = config
  fileInfo.value = info
  await refreshProvidersAndSelection()
  hydrateDraftsFromConfig()
}

const ensureDefaultConfigReady = async () => {
  try {
    await loadConfig()
  } catch {
    await invoke<string>('generate_default_config')
    await loadConfig()
  }
  await refreshChannelExtensionStatus()
}

const refreshModels = async ({ silent = false }: { silent?: boolean } = {}) => {
  const apiKey = providerApiKey.value.trim()
  if (!apiKey) {
    fetchedModels.value = []
    if (!silent) infoMessage.value = '填写 API Key 后可自动拉取模型列表，也可以直接手动填写模型 ID。'
    return
  }
  const baseUrl = resolvedProviderBaseUrl.value
  if (!baseUrl) {
    fetchedModels.value = []
    if (!silent) infoMessage.value = '请先填写服务商 Base URL，再尝试自动拉取模型列表。'
    return
  }
  loadingModels.value = true
  errorMessage.value = ''
  if (!silent) infoMessage.value = ''
  try {
    fetchedModels.value = await invoke<string[]>('fetch_provider_models', {
      baseUrl,
      apiKey,
    })
    modelOptionsRequestKey.value = currentModelOptionsRequestKey.value
  } catch (error) {
    fetchedModels.value = []
    modelOptionsRequestKey.value = currentModelOptionsRequestKey.value
    if (!silent) infoMessage.value = `模型列表暂时没有自动拉取到：${error}。你可以继续手动填写模型 ID。`
  } finally {
    loadingModels.value = false
  }
}

const ensureModelOptionsLoaded = async () => {
  const requestKey = currentModelOptionsRequestKey.value
  if (!requestKey || loadingModels.value || modelOptionsRequestKey.value === requestKey) return
  await refreshModels({ silent: true })
}

const openModelDropdown = async () => {
  showModelDropdown.value = true
  await ensureModelOptionsLoaded()
}

const saveModelStep = async () => {
  const apiKey = providerApiKey.value.trim()
  const modelId = modelQuery.value.trim()
  if (!apiKey) throw new Error('请先填写 API Key')
  if (!modelId) throw new Error('请先选择或输入模型 ID')
  if (!currentConfig.value) throw new Error('配置文件未加载')

  setBusy('正在写入模型配置...')
  try {
    const preset = isCustomProvider.value
      ? createQuickSetupCustomProviderPreset({
          providerName: customProviderName.value,
          baseUrl: customProviderBaseUrl.value,
          selectedModelId: modelId,
        })
      : currentProviderPreset.value
    if (isCustomProvider.value) {
      if (!customProviderName.value.trim()) throw new Error('请先填写自定义服务商名称')
      if (!customProviderBaseUrl.value.trim()) throw new Error('请先填写自定义服务商 Base URL')
    }
    const nextConfig = applyQuickSetupModelPreset(currentConfig.value, preset, apiKey, modelId)

    currentConfig.value = nextConfig
    await saveConfigToDisk()
    await refreshProvidersAndSelection()
    markStepSaved('model')
    infoMessage.value = `主模型已更新为 ${resolvedProviderDisplayName.value} / ${modelId}`
    props.showToast('success', '模型配置已保存')
    moveNext()
  } finally {
    clearBusy()
  }
}

const ensureSelectedChannelExtension = async () => {
  if (extensionInstalled.value) return
  await invoke<string>('install_channel_extension', { channelId: selectedChannelId.value })
  await refreshChannelExtensionStatus()
}

const ensureQuickSetupChannelNode = (channelId: QuickSetupChannelId) => {
  if (!currentConfig.value) throw new Error('配置文件未加载')
  const root = asRecord(currentConfig.value)
  const channels = asRecord(root.channels)
  root.channels = channels
  const configKey = getChannelConfigKey(channelId)
  const node = asRecord(channels[configKey])
  channels[configKey] = node
  return node
}

const resetManagedQuickSetupChannels = async () => {
  if (!currentConfig.value) throw new Error('配置文件未加载')
  currentConfig.value = clearQuickSetupManagedChannels(currentConfig.value)
  await saveConfigToDisk()
}

const saveChannelStep = async () => {
  setBusy('正在写入通信渠道配置...')
  try {
    await resetManagedQuickSetupChannels()

    if (selectedChannelId.value === 'feishu') {
      const appId = channelIdValue.value.trim()
      const appSecret = channelSecretValue.value.trim()
      if (!appId || !appSecret) throw new Error('请填写完整的飞书 App ID 与 App Secret')
      await ensureSelectedChannelExtension()
      mergeFeishuChannelConfig(currentConfig.value as Record<string, unknown>, {
        appId,
        appSecret,
        enabled: true,
        domain: 'feishu',
        connectionMode: 'websocket',
      })
      await saveConfigToDisk()
    } else if (selectedChannelId.value === 'dingtalk') {
      const clientId = channelIdValue.value.trim()
      const clientSecret = channelSecretValue.value.trim()
      if (!clientId || !clientSecret) throw new Error('请填写完整的钉钉 Client ID 与 Client Secret')
      await ensureSelectedChannelExtension()
      await invoke<string>('set_dingtalk_channel_config', {
        clientId,
        clientSecret,
        enabled: true,
      })
    } else if (selectedChannelId.value === 'wecom') {
      const botId = channelIdValue.value.trim()
      const secret = channelSecretValue.value.trim()
      if (!botId || !secret) throw new Error('请填写完整的企业微信 Bot ID 与 Secret')
      await ensureSelectedChannelExtension()
      const node = ensureQuickSetupChannelNode('wecom')
      node.enabled = true
      node.botId = botId
      node.secret = secret
      await saveConfigToDisk()
    } else if (selectedChannelId.value === 'qq') {
      const appId = channelIdValue.value.trim()
      const clientSecret = channelSecretValue.value.trim()
      if (!appId || !clientSecret) throw new Error('请填写完整的 QQ App ID 与 App Secret')
      await ensureSelectedChannelExtension()
      const node = ensureQuickSetupChannelNode('qq')
      node.enabled = true
      node.appId = appId
      node.clientSecret = clientSecret
      delete node.token
      delete node.clientSecretFile
      await saveConfigToDisk()
    } else {
      throw new Error(`暂不支持的快速引导渠道：${selectedChannelId.value}`)
    }

    await loadConfig()
    markStepSaved('channel')
    infoMessage.value = `${currentChannelPreset.value.name} 已完成快速接入`
    props.showToast('success', '通信渠道配置已保存')
    moveNext()
  } finally {
    clearBusy()
  }
}

const openManualOnboard = async () => {
  try {
    await invoke<string>('open_terminal_with_command', {
      command: 'openclaw onboard --install-daemon',
    })
    infoMessage.value = '已打开终端执行 openclaw onboard --install-daemon'
    errorMessage.value = ''
    props.showToast('success', '已打开终端执行手动配置命令')
  } catch (error) {
    const message = formatGatewayInstallError(`打开终端失败：${error}`)
    errorMessage.value = message
    props.showToast('error', message)
  }
}

const relaunchAsAdmin = async () => {
  adminRelaunching.value = true
  adminRelaunchMessage.value = ''

  try {
    persistQuickSetupSession('awaiting_admin_relaunch')
    adminRelaunchMessage.value = await invoke<string>('relaunch_as_admin')
  } catch (error) {
    persistQuickSetupSession('in_progress')
    errorMessage.value = formatGatewayInstallError(`管理员重启失败：${error}`)
  } finally {
    adminRelaunching.value = false
  }
}

const closeQuickSetup = () => {
  if (busy.value || adminRelaunching.value) return
  stopQuickSetupSessionPersistence()
  emit('close')
}

const installGatewayAndEnterDashboard = async () => {
  if (!hasReadyPrimaryModel.value) {
    stepIndex.value = 0
    throw new Error('进入工作台前需要先完成有效主模型配置')
  }
  if (!currentConfig.value) {
    throw new Error('配置文件未加载')
  }

  currentConfig.value = sanitizeQuickSetupChannelConfig(currentConfig.value)
  currentConfig.value = applyQuickSetupGatewayOptions(currentConfig.value, {
    browserDefaultProfileEnabled: browserDefaultProfileEnabled.value,
    toolsFullProfileEnabled: toolsFullProfileEnabled.value,
  })

  try {
    setBusy('正在保存网关关键配置...')
    await saveConfigToDisk()

    setBusy('正在安装并启动网关...')
    await invoke<string>('install_gateway_service')
    if (props.systemOs !== 'windows') {
      setBusy('正在启动网关服务...')
      await invoke<string>('start_gateway')
    }
    setBusy('正在等待网关健康检查通过...')
    const ready = await waitForGatewayReady(() => invoke<boolean>('health_check_gateway'), DEFAULT_GATEWAY_READY_OPTIONS)
    if (!ready) {
      throw new Error('网关在预期时间内未完成启动，请稍后重试')
    }

    stopQuickSetupSessionPersistence()
    markStepSaved('gateway')
    infoMessage.value = '快速引导完成，正在进入工作台。'
    props.showToast('success', '网关已启动，正在进入工作台')
    emit('complete')
  } finally {
    clearBusy()
  }
}

const handlePrimaryAction = async () => {
  errorMessage.value = ''
  infoMessage.value = ''
  try {
    if (currentStep.value.id === 'model') {
      await saveModelStep()
      return
    }
    if (currentStep.value.id === 'channel') {
      await saveChannelStep()
      return
    }
    await installGatewayAndEnterDashboard()
  } catch (error) {
    const message = formatGatewayInstallError(error)
    errorMessage.value = message
    props.showToast('error', message)
  }
}

watch(selectedProviderId, () => {
  fetchedModels.value = []
  modelOptionsRequestKey.value = ''
  showModelDropdown.value = false
  const providerName = isCustomProvider.value ? customProviderName.value.trim() : currentProviderPreset.value.name
  providerApiKey.value = currentConfig.value?.models?.providers?.[providerName]?.apiKey || ''
  if (isCustomProvider.value) {
    modelSelectionMode.value = 'manual'
  } else {
    modelSelectionMode.value = 'auto'
  }
})

watch(currentModelOptionsRequestKey, (nextKey, previousKey) => {
  if (nextKey === previousKey) return
  fetchedModels.value = []
  modelOptionsRequestKey.value = ''
})

watch(selectedChannelId, () => {
  channelIdValue.value = ''
  channelSecretValue.value = ''
  errorMessage.value = ''
})

const handleClickOutside = (event: MouseEvent) => {
  const target = event.target as HTMLElement | null
  if (!target?.closest('.quick-setup-model-dropdown-container')) {
    showModelDropdown.value = false
  }
}

watch(
  [
    stepIndex,
    savedStepIds,
    selectedProviderId,
    providerApiKey,
    modelQuery,
    modelSelectionMode,
    customProviderName,
    customProviderBaseUrl,
    selectedChannelId,
    channelIdValue,
    channelSecretValue,
    browserDefaultProfileEnabled,
    toolsFullProfileEnabled,
  ],
  () => {
    persistQuickSetupSession('in_progress')
  },
  { deep: true }
)

onMounted(async () => {
  document.addEventListener('click', handleClickOutside)
  setBusy('正在准备快速引导...')
  try {
    await ensureDefaultConfigReady()
    restoreQuickSetupSessionState()
  } catch (error) {
    errorMessage.value = `初始化快速引导失败：${error}`
  } finally {
    clearBusy()
  }
})

onBeforeUnmount(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<template>
  <div class="oc-page-root h-full min-h-0">
    <div class="grid h-full min-h-0 gap-4 xl:grid-cols-[280px,minmax(0,1fr)]">
      <aside class="oc-panel oc-quick-setup-sidebar flex min-h-0 flex-col p-4">
        <div class="oc-quick-setup-hero rounded-[18px] border p-4" style="border-color: color-mix(in srgb, var(--oc-accent) 10%, var(--oc-card-border));">
          <div class="flex items-center gap-3">
            <div class="flex h-11 w-11 items-center justify-center rounded-full" style="background: color-mix(in srgb, var(--oc-accent) 10%, transparent); color: var(--oc-accent);">
              <Sparkles class="h-5 w-5" />
            </div>
            <div>
              <p class="text-base font-semibold" style="color: var(--oc-text-primary);">快速引导</p>
              <p class="text-xs" style="color: var(--oc-text-secondary);">首次配置入口</p>
            </div>
          </div>
          <p class="mt-3 text-sm leading-6" style="color: var(--oc-text-secondary);">三步完成模型、渠道和网关配置。</p>
        </div>

        <div class="mt-4 space-y-2">
          <button
            v-for="(step, index) in QUICK_SETUP_STEPS"
            :key="step.id"
            type="button"
            class="oc-quick-setup-step w-full rounded-[14px] border px-4 py-3 text-left transition-all"
            :style="{
              borderColor: index === stepIndex ? 'color-mix(in srgb, var(--oc-accent) 18%, var(--oc-card-border))' : 'var(--oc-card-border)',
              background: index === stepIndex ? 'color-mix(in srgb, var(--oc-accent-soft) 28%, var(--oc-card) 72%)' : 'var(--oc-card-elevated)',
            }"
            @click="stepIndex = index"
          >
            <div class="flex items-start justify-between gap-3">
              <div>
                <p class="text-sm font-medium" style="color: var(--oc-text-primary);">{{ index + 1 }}. {{ step.title }}</p>
                <p class="mt-1 text-xs leading-5" style="color: var(--oc-text-muted);">{{ step.subtitle }}</p>
              </div>
              <CheckCircle2
                class="h-4 w-4 shrink-0"
                :style="{ color: savedStepIds.includes(step.id) ? 'var(--oc-success)' : 'var(--oc-text-quiet)' }"
              />
            </div>
          </button>
        </div>

        <div class="mt-auto rounded-[14px] border p-3 text-xs leading-5" style="border-color: color-mix(in srgb, var(--oc-accent) 10%, var(--oc-card-border)); background: color-mix(in srgb, var(--oc-accent-soft) 18%, var(--oc-card) 82%); color: var(--oc-text-secondary);">
          <p>当前主模型：<span style="color: var(--oc-text-primary);">{{ primaryModelPath || '未设置' }}</span></p>
          <p class="mt-1">当前渠道：<span style="color: var(--oc-text-primary);">{{ currentChannelPreset.name }}</span></p>
          <p class="mt-1">当前配置文件：<span style="color: var(--oc-text-primary);">{{ fileInfo?.fileName || 'openclaw.json' }}</span></p>
        </div>
      </aside>

      <section class="oc-panel oc-quick-setup-main flex min-h-0 flex-col overflow-hidden p-5">
        <div class="flex shrink-0 items-start justify-between gap-4 border-b pb-3" style="border-color: var(--oc-divider-soft);">
          <div>
            <div class="flex items-center gap-2 text-xs font-medium uppercase tracking-[0.18em]" style="color: var(--oc-text-muted);">
              <component :is="currentStep.id === 'model' ? Bot : currentStep.id === 'channel' ? MessageSquareMore : Rocket" class="h-4 w-4" />
              Step {{ stepIndex + 1 }}
            </div>
            <h2 class="mt-1.5 text-[28px] font-semibold leading-tight" style="color: var(--oc-text-primary);">{{ currentStep.title }}</h2>
          </div>
          <Button v-if="showCloseAction" variant="outline" :disabled="busy || adminRelaunching" @click="closeQuickSetup">
            <X class="h-4 w-4" />
            关闭
          </Button>
        </div>

        <div v-if="errorMessage" class="mt-4 rounded-[14px] border px-4 py-3 text-sm whitespace-pre-line" style="border-color: color-mix(in srgb, var(--oc-danger) 32%, transparent); background: color-mix(in srgb, var(--oc-danger) 9%, transparent); color: var(--oc-danger);">
          {{ errorMessage }}
        </div>
        <div v-if="canRelaunchAsAdmin" class="mt-3 flex flex-col items-start gap-3">
          <Button :disabled="adminRelaunching" @click="relaunchAsAdmin">
            <ShieldCheck class="h-4 w-4" />
            {{ adminRelaunching ? '正在请求管理员权限...' : '以管理员身份重启' }}
          </Button>
          <p v-if="adminRelaunchMessage" class="text-sm whitespace-pre-line" style="color: var(--oc-text-secondary);">
            {{ adminRelaunchMessage }}
          </p>
        </div>
        <div v-else-if="infoMessage || busyMessage" class="mt-4 rounded-[14px] border px-4 py-3 text-sm" style="border-color: color-mix(in srgb, var(--oc-accent) 14%, transparent); background: color-mix(in srgb, var(--oc-accent) 4%, transparent); color: var(--oc-text-secondary);">
          <span v-if="busy" class="inline-flex items-center gap-2"><Loader2 class="h-4 w-4 animate-spin" />{{ busyMessage }}</span>
          <span v-else>{{ infoMessage }}</span>
        </div>

        <div class="mt-4 flex min-h-0 flex-1 flex-col overflow-hidden">
          <div v-if="currentStep.id === 'model'" class="grid min-h-0 flex-1 gap-4 overflow-hidden xl:grid-cols-[260px,minmax(0,1fr)]">
            <div class="flex min-h-0 flex-col rounded-[18px] border p-4" style="border-color: color-mix(in srgb, var(--oc-accent) 10%, var(--oc-card-border)); background: var(--oc-card);">
              <div class="flex items-center justify-between gap-3">
                <p class="text-sm font-medium" style="color: var(--oc-text-primary);">模型服务商</p>
                <span class="rounded-full px-2 py-0.5 text-[11px]" style="background: color-mix(in srgb, var(--oc-accent-soft) 55%, transparent); color: var(--oc-accent);">
                  {{ QUICK_SETUP_PROVIDER_PRESETS.length }} 个选项
                </span>
              </div>
              <div class="mt-4 min-h-0 flex-1 space-y-2 overflow-y-auto pr-1">
                <button
                  v-for="preset in QUICK_SETUP_PROVIDER_PRESETS"
                  :key="preset.id"
                  type="button"
                  class="w-full rounded-[14px] border px-4 py-3 text-left transition-all"
                  :style="{
                    borderColor: selectedProviderId === preset.id ? 'var(--oc-card-border-strong)' : 'var(--oc-card-border)',
                    background: selectedProviderId === preset.id ? 'var(--oc-item-active)' : 'var(--oc-card-elevated)',
                  }"
                  @click="selectedProviderId = preset.id"
                >
                  <div class="flex items-start justify-between gap-3">
                    <div class="min-w-0 flex-1">
                      <p class="text-sm font-semibold" style="color: var(--oc-text-primary);">{{ preset.displayName }}</p>
                      <p class="mt-1 text-xs leading-5" style="color: var(--oc-text-secondary);">{{ preset.description }}</p>
                    </div>
                    <span v-if="preset.isCustom" class="rounded-full px-2 py-0.5 text-[11px]" style="background: color-mix(in srgb, var(--oc-warning) 16%, transparent); color: var(--oc-warning);">
                      自定义
                    </span>
                  </div>
                </button>
              </div>
            </div>

            <div class="min-h-0 overflow-y-auto pr-1">
              <div class="space-y-4">
                <div class="rounded-[18px] border p-4" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated);">
                  <div class="flex items-start justify-between gap-4">
                    <div>
                      <p class="text-sm font-semibold" style="color: var(--oc-text-primary);">{{ resolvedProviderDisplayName }}</p>
                    </div>
                  </div>
                  <p v-if="!isCustomProvider" class="mt-3 text-xs leading-5" style="color: var(--oc-text-muted);">预设服务商的 Base URL 已内置，不需要在快速引导中额外填写。</p>
                </div>

                <div v-if="isCustomProvider" class="rounded-[18px] border p-4" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated);">
                  <div class="grid gap-4 lg:grid-cols-2">
                    <div>
                      <label class="mb-2 block text-sm font-medium" style="color: var(--oc-text-secondary);">服务商名称</label>
                      <Input v-model="customProviderName" placeholder="例如 custom-openai" autocomplete="off" />
                    </div>
                    <div>
                      <label class="mb-2 block text-sm font-medium" style="color: var(--oc-text-secondary);">Base URL</label>
                      <Input v-model="customProviderBaseUrl" placeholder="https://example.com/v1" autocomplete="off" />
                    </div>
                  </div>
                </div>

                <div class="rounded-[18px] border p-4" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated);">
                  <label class="mb-2 block text-sm font-medium" style="color: var(--oc-text-secondary);">API Key</label>
                  <Input v-model="providerApiKey" type="password" placeholder="输入服务商 API Key" autocomplete="off" />
                </div>

                <div class="rounded-[18px] border p-4" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated);">
                  <div class="mb-2 flex items-center justify-between gap-3">
                    <label class="block text-sm font-medium" style="color: var(--oc-text-secondary);">主模型</label>
                    <span class="rounded-full px-2 py-0.5 text-[11px]" style="background: color-mix(in srgb, var(--oc-card) 88%, transparent); color: var(--oc-text-muted);">
                      {{ modelOptionBadgeLabel }}
                    </span>
                  </div>
                  <div class="relative quick-setup-model-dropdown-container">
                    <Input
                      v-model="modelQuery"
                      :placeholder="modelInputPlaceholder"
                      autocomplete="off"
                      autocorrect="off"
                      autocapitalize="off"
                      spellcheck="false"
                      lang="en"
                      @focus="() => { void openModelDropdown() }"
                      @click="() => { void openModelDropdown() }"
                    />
                    <div
                      v-if="shouldShowModelDropdown"
                      class="oc-dropdown-menu absolute inset-x-0 top-full z-10 mt-1 max-h-48 overflow-auto"
                      @click.stop
                    >
                      <div v-if="loadingModels" class="oc-dropdown-empty">加载中...</div>
                      <template v-else>
                        <button
                          v-for="item in filteredModelOptions"
                          :key="item.id"
                          type="button"
                          class="oc-dropdown-item cursor-pointer text-sm"
                          @click="selectModelFromDropdown(item.id)"
                        >
                          {{ item.name }}
                        </button>
                        <p v-if="filteredModelOptions.length === 0" class="oc-dropdown-empty">无匹配结果</p>
                      </template>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div v-else-if="currentStep.id === 'channel'" class="grid min-h-0 flex-1 gap-4 overflow-hidden xl:grid-cols-[260px,minmax(0,1fr)]">
            <div class="flex min-h-0 flex-col rounded-[18px] border p-4" style="border-color: color-mix(in srgb, var(--oc-accent) 10%, var(--oc-card-border)); background: var(--oc-card);">
              <p class="text-sm font-medium" style="color: var(--oc-text-primary);">通信渠道</p>

              <div class="mt-4 min-h-0 flex-1 space-y-2 overflow-y-auto pr-1">
                <button
                  v-for="channel in QUICK_SETUP_CHANNEL_PRESETS"
                  :key="channel.id"
                  type="button"
                  class="w-full rounded-[14px] border px-4 py-3 text-left transition-all"
                  :style="{
                    borderColor: selectedChannelId === channel.id ? 'var(--oc-card-border-strong)' : 'var(--oc-card-border)',
                    background: selectedChannelId === channel.id ? 'var(--oc-item-active)' : 'var(--oc-card-elevated)',
                  }"
                  @click="selectedChannelId = channel.id"
                >
                  <div class="flex items-center justify-between gap-3">
                    <p class="text-sm font-semibold" style="color: var(--oc-text-primary);">{{ channel.name }}</p>
                    <span class="rounded-full px-2 py-0.5 text-[11px]" style="background: color-mix(in srgb, var(--oc-card) 88%, transparent); color: var(--oc-text-muted);">
                      {{ isChannelPluginInstalled(channelExtensionStatus, channel.id) ? '已安装插件' : '待安装插件' }}
                    </span>
                  </div>
                </button>
              </div>
            </div>

            <div class="min-h-0 overflow-y-auto pr-1">
              <div class="space-y-4">
                <div class="rounded-[18px] border p-4" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated);">
                  <p class="text-sm font-semibold" style="color: var(--oc-text-primary);">{{ currentChannelPreset.name }}</p>
                </div>

                <div class="rounded-[18px] border p-4" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated);">
                  <label class="mb-2 block text-sm font-medium" style="color: var(--oc-text-secondary);">{{ currentChannelPreset.placeholderLabel }}</label>
                  <Input
                    v-model="channelIdValue"
                    type="text"
                    :placeholder="`输入${currentChannelPreset.placeholderLabel}`"
                    autocomplete="off"
                  />
                  <p v-if="selectedChannelId === 'qq'" class="mt-2 text-xs leading-5" style="color: var(--oc-text-muted);">
                    QQ 渠道会分别写入 <code>channels.qqbot.appId</code> 与 <code>channels.qqbot.clientSecret</code>。
                  </p>
                </div>

                <div
                  v-if="currentChannelPreset.secretLabel"
                  class="rounded-[18px] border p-4"
                  style="border-color: var(--oc-card-border); background: var(--oc-card-elevated);"
                >
                  <label class="mb-2 block text-sm font-medium" style="color: var(--oc-text-secondary);">{{ currentChannelPreset.secretLabel }}</label>
                  <Input v-model="channelSecretValue" type="password" :placeholder="`输入${currentChannelPreset.secretLabel}`" autocomplete="off" />
                </div>
              </div>
            </div>
          </div>

          <div v-else class="grid min-h-0 flex-1 gap-4 lg:grid-cols-[minmax(0,0.94fr),minmax(0,1.06fr)]">
            <div class="rounded-[16px] border p-4" style="border-color: color-mix(in srgb, var(--oc-accent) 8%, var(--oc-card-border)); background: color-mix(in srgb, var(--oc-accent-soft) 16%, var(--oc-card) 84%);">
              <p class="text-sm font-semibold" style="color: var(--oc-text-primary);">当前配置摘要</p>
              <div class="mt-3 space-y-2 text-sm" style="color: var(--oc-text-secondary);">
                <p>主模型：<span style="color: var(--oc-text-primary);">{{ primaryModelPath || '未配置' }}</span></p>
                <p>通信渠道：<span style="color: var(--oc-text-primary);">{{ currentChannelPreset.name }}</span></p>
                <p>渠道凭据：<span style="color: var(--oc-text-primary);">{{ selectedChannelSummary || '未配置' }}</span></p>
                <p>浏览器配置：<span style="color: var(--oc-text-primary);">{{ browserProfileStatusText }}</span></p>
                <p>工具配置：<span style="color: var(--oc-text-primary);">{{ toolsProfileStatusText }}</span></p>
              </div>
            </div>

            <div class="rounded-[16px] border p-4" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated);">
              <div class="flex items-start justify-between gap-3">
                <div>
                  <p class="text-sm font-semibold" style="color: var(--oc-text-primary);">关键配置</p>
                  <p class="mt-2 text-sm leading-6" style="color: var(--oc-text-secondary);">这两项为可选增强项，仅在开启时写入配置文件。</p>
                </div>
                <span class="rounded-full border px-3 py-1 text-xs" style="border-color: color-mix(in srgb, var(--oc-accent) 14%, var(--oc-card-border)); color: var(--oc-accent); background: color-mix(in srgb, var(--oc-accent-soft) 60%, transparent);">可选项</span>
              </div>

              <div class="mt-4 space-y-3">
                <div class="oc-quick-toggle-card rounded-[12px] border p-4">
                  <div class="flex items-start justify-between gap-4">
                    <div class="min-w-0 flex-1">
                      <div class="flex flex-wrap items-center gap-2">
                        <p class="text-sm font-medium" style="color: var(--oc-text-primary);">浏览器默认配置</p>
                        <span class="oc-quick-toggle-state text-[11px]" :class="{ 'is-on': browserDefaultProfileEnabled }">
                          {{ browserDefaultProfileEnabled ? '已启用' : '默认' }}
                        </span>
                      </div>
                      <p class="mt-2 text-xs leading-5" style="color: var(--oc-text-muted);">
                        自动写入浏览器默认 Profile，便于后续调试与隔离使用。
                      </p>
                    </div>
                    <div class="flex shrink-0 items-center">
                      <button
                        type="button"
                        aria-label="toggle-quick-setup-browser-default-profile"
                        :aria-checked="browserDefaultProfileEnabled"
                        class="oc-quick-toggle"
                        :class="{ 'is-on': browserDefaultProfileEnabled }"
                        :disabled="busy"
                        @click="browserDefaultProfileEnabled = !browserDefaultProfileEnabled"
                      >
                        <span class="oc-quick-toggle-thumb" />
                      </button>
                    </div>
                  </div>
                </div>

                <div class="oc-quick-toggle-card rounded-[12px] border p-4">
                  <div class="flex items-start justify-between gap-4">
                    <div class="min-w-0 flex-1">
                      <div class="flex flex-wrap items-center gap-2">
                        <p class="text-sm font-medium" style="color: var(--oc-text-primary);">完整工具能力</p>
                        <span class="oc-quick-toggle-state text-[11px]" :class="{ 'is-on': toolsFullProfileEnabled }">
                          {{ toolsFullProfileEnabled ? '已启用' : '默认' }}
                        </span>
                      </div>
                      <p class="mt-2 text-xs leading-5" style="color: var(--oc-text-muted);">
                        写入完整工具配置，便于后续使用浏览器、自动化与调试能力。
                      </p>
                    </div>
                    <div class="flex shrink-0 items-center">
                      <button
                        type="button"
                        aria-label="toggle-quick-setup-tools-full-profile"
                        :aria-checked="toolsFullProfileEnabled"
                        class="oc-quick-toggle"
                        :class="{ 'is-on': toolsFullProfileEnabled }"
                        :disabled="busy"
                        @click="toolsFullProfileEnabled = !toolsFullProfileEnabled"
                      >
                        <span class="oc-quick-toggle-thumb" />
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="mt-4 flex shrink-0 items-center justify-between gap-3 border-t pt-3" style="border-color: var(--oc-divider-soft);">
          <Button variant="outline" :disabled="stepIndex === 0 || busy" @click="movePrev">
            <ChevronLeft class="h-4 w-4" />
            上一步
          </Button>

          <div class="flex items-center gap-2">
            <Button v-if="canSkipQuickSetupStep(currentStep.id)" variant="ghost" :disabled="busy" @click="skipCurrentStep">
              跳过
            </Button>
            <Button :disabled="busy || (currentStep.id === 'gateway' && !hasReadyPrimaryModel)" @click="handlePrimaryAction">
              <Loader2 v-if="busy" class="h-4 w-4 animate-spin" />
              <span v-else>{{ primaryButtonLabel }}</span>
            </Button>
            <Button v-if="currentStep.id === 'gateway'" variant="outline" :disabled="busy" @click="openManualOnboard">
              <ShieldCheck class="h-4 w-4" />
              手动配置
            </Button>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.oc-quick-setup-sidebar {
  background:
    radial-gradient(520px 280px at 0% 0%, color-mix(in srgb, var(--oc-accent-soft) 36%, transparent), transparent 72%),
    linear-gradient(180deg, color-mix(in srgb, var(--oc-card) 98%, var(--oc-accent-soft) 2%), var(--oc-card));
}

.oc-quick-setup-main {
  background:
    radial-gradient(680px 300px at 100% 0%, color-mix(in srgb, var(--oc-accent-soft) 28%, transparent), transparent 68%),
    linear-gradient(180deg, color-mix(in srgb, var(--oc-card) 99%, var(--oc-accent-soft) 1%), var(--oc-card));
}

.oc-quick-setup-hero {
  background:
    linear-gradient(135deg, color-mix(in srgb, var(--oc-accent-soft) 22%, var(--oc-card) 78%), var(--oc-card));
  box-shadow: 0 8px 18px color-mix(in srgb, var(--oc-accent) 4%, transparent);
}

.oc-quick-setup-step:hover {
  transform: translateY(-1px);
  box-shadow: 0 8px 18px color-mix(in srgb, var(--oc-accent) 4%, transparent);
}

.oc-quick-toggle-card {
  border-color: var(--oc-card-border);
  background: color-mix(in srgb, var(--oc-card) 88%, transparent);
}

.oc-quick-toggle-state {
  display: inline-flex;
  align-items: center;
  border-radius: 9999px;
  border: 1px solid var(--oc-card-border);
  padding: 2px 8px;
  color: var(--oc-text-muted);
  background: color-mix(in srgb, var(--oc-card) 92%, transparent);
}

.oc-quick-toggle-state.is-on {
  border-color: color-mix(in srgb, var(--oc-accent) 16%, var(--oc-card-border));
  color: var(--oc-accent);
  background: color-mix(in srgb, var(--oc-accent-soft) 75%, transparent);
}

.oc-quick-toggle {
  position: relative;
  display: inline-flex;
  height: 26px;
  width: 46px;
  flex-shrink: 0;
  align-items: center;
  border-radius: 9999px;
  border: 1px solid var(--oc-card-border);
  background: color-mix(in srgb, var(--oc-card) 92%, transparent);
  transition: background-color 160ms ease, border-color 160ms ease, box-shadow 160ms ease;
}

.oc-quick-toggle:hover {
  border-color: color-mix(in srgb, var(--oc-accent) 12%, var(--oc-card-border));
}

.oc-quick-toggle.is-on {
  border-color: color-mix(in srgb, var(--oc-accent) 20%, var(--oc-card-border));
  background: color-mix(in srgb, var(--oc-accent-soft) 85%, transparent);
  box-shadow: 0 0 0 1px color-mix(in srgb, var(--oc-accent) 10%, transparent);
}

.oc-quick-toggle:disabled {
  cursor: not-allowed;
  opacity: 0.66;
}

.oc-quick-toggle-thumb {
  height: 18px;
  width: 18px;
  margin-left: 3px;
  border-radius: 9999px;
  border: 1px solid color-mix(in srgb, var(--oc-card-border) 88%, white 12%);
  background: var(--oc-card);
  box-shadow: 0 1px 2px rgba(15, 23, 42, 0.12);
  transition: transform 160ms ease;
}

.oc-quick-toggle.is-on .oc-quick-toggle-thumb {
  transform: translateX(20px);
}
</style>
