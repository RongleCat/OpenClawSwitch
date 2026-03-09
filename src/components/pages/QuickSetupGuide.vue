<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import {
  Bot,
  CheckCircle2,
  ChevronDown,
  ChevronLeft,
  Loader2,
  MessageSquareMore,
  RefreshCw,
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
  clearQuickSetupManagedChannels,
  QUICK_SETUP_CHANNEL_PRESETS,
  QUICK_SETUP_PROVIDER_PRESETS,
  QUICK_SETUP_STEPS,
  canSkipQuickSetupStep,
  findProviderPreset,
  sanitizeQuickSetupChannelConfig,
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
  type QuickSetupSessionStatus,
} from '../../domain/quickSetupSession'
import { resolveDingtalkChannelNode } from '../../domain/dingtalkPlugin'
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

interface ChannelExtensionStatus {
  feishuInstalled: boolean
  dingtalkInstalled: boolean
}

interface ModelOption {
  id: string
  name: string
}

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
const channelExtensionStatus = ref<ChannelExtensionStatus>({ feishuInstalled: false, dingtalkInstalled: false })

const selectedProviderId = ref<QuickSetupProviderId>('dashscope-coding')
const providerApiKey = ref('')
const modelQuery = ref('')
const fetchedModels = ref<string[]>([])
const loadingModels = ref(false)

const selectedChannelId = ref<QuickSetupChannelId>('feishu')
const channelIdValue = ref('')
const channelSecretValue = ref('')
const browserDefaultProfileEnabled = ref(false)
const toolsFullProfileEnabled = ref(false)

const savedStepIds = ref<QuickSetupStepId[]>([])
const restoringSession = ref(false)

const currentStep = computed(() => QUICK_SETUP_STEPS[stepIndex.value])
const canRelaunchAsAdmin = computed(() => isAdminRequiredGatewayInstallError(errorMessage.value))
const currentProviderPreset = computed(() => findProviderPreset(selectedProviderId.value)!)
const currentChannelPreset = computed(
  () => QUICK_SETUP_CHANNEL_PRESETS.find((item) => item.id === selectedChannelId.value) ?? QUICK_SETUP_CHANNEL_PRESETS[0]
)
const hasReadyPrimaryModel = computed(() => {
  const primary = modelSelection.value.primary
  return Boolean(primary && !isPrimaryModelPlaceholder(primary))
})
const isExtensionChannel = computed(() => selectedChannelId.value === 'feishu' || selectedChannelId.value === 'dingtalk')
const channelNeedsPrimaryField = computed(() => isExtensionChannel.value || selectedChannelId.value === 'slack')
const extensionInstalled = computed(() => {
  if (selectedChannelId.value === 'feishu') return channelExtensionStatus.value.feishuInstalled
  if (selectedChannelId.value === 'dingtalk') return channelExtensionStatus.value.dingtalkInstalled
  return true
})
const modelOptions = computed<ModelOption[]>(() => {
  const unique = new Map<string, ModelOption>()
  for (const item of currentProviderPreset.value.suggestedModels) {
    unique.set(item.id, item)
  }
  for (const item of fetchedModels.value) {
    if (!unique.has(item)) {
      unique.set(item, { id: item, name: item })
    }
  }
  return Array.from(unique.values())
})
const filteredModelOptions = computed(() => {
  const query = modelQuery.value.trim().toLowerCase()
  if (!query) return modelOptions.value
  return modelOptions.value.filter((item) => item.id.toLowerCase().includes(query) || item.name.toLowerCase().includes(query))
})
const primaryButtonLabel = computed(() => {
  if (busy.value) return busyMessage.value || '处理中...'
  if (currentStep.value.id === 'model') return '保存模型并下一步'
  if (currentStep.value.id === 'channel') return '保存渠道并下一步'
  return '安装网关并进入工作台'
})
const currentProviderApiKey = computed(() => {
  const providerName = currentProviderPreset.value.name
  return currentConfig.value?.models?.providers?.[providerName]?.apiKey || ''
})
const primaryModelPath = computed(() => modelSelection.value.primary || '')
const maskSecret = (value: string) => {
  const trimmed = value.trim()
  if (!trimmed) return ''
  if (trimmed.length <= 8) return trimmed
  return trimmed.slice(0, 4) + '...' + trimmed.slice(-4)
}
const selectedChannelSummary = computed(() => {
  if (isExtensionChannel.value) {
    return channelIdValue.value.trim() || maskSecret(channelSecretValue.value)
  }
  if (selectedChannelId.value === 'slack') {
    return maskSecret(channelSecretValue.value) || channelIdValue.value.trim()
  }
  return maskSecret(channelSecretValue.value)
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
  if (restoringSession.value) return
  const currentStepId = QUICK_SETUP_STEPS[stepIndex.value]?.id ?? 'model'
  saveQuickSetupSession(createQuickSetupSessionSnapshot({
    status,
    stepId: currentStepId,
    savedStepIds: savedStepIds.value,
    selectedProviderId: selectedProviderId.value,
    providerApiKey: providerApiKey.value,
    modelQuery: modelQuery.value,
    selectedChannelId: selectedChannelId.value,
    channelIdValue: channelIdValue.value,
    channelSecretValue: channelSecretValue.value,
    browserDefaultProfileEnabled: browserDefaultProfileEnabled.value,
    toolsFullProfileEnabled: toolsFullProfileEnabled.value,
  }))
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
  channelExtensionStatus.value = await invoke<ChannelExtensionStatus>('get_channel_extension_status')
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
    const matchedPreset = QUICK_SETUP_STEPS && findProviderPreset(selectedProviderId.value)
    void matchedPreset
    const preset = ['dashscope-coding', 'tencent-coding', 'deepseek', 'dashscope', 'hunyuan']
      .map((id) => findProviderPreset(id))
      .find((item) => item && (item.name === providerName || item.baseUrl === providerConfig?.baseUrl))
    if (preset) {
      selectedProviderId.value = preset.id
      modelQuery.value = modelId || currentProviderPreset.value.suggestedModels[0]?.id || ''
    }
    markStepSaved('model')
  } else if (!modelQuery.value) {
    modelQuery.value = currentProviderPreset.value.suggestedModels[0]?.id || ''
  }

  const channels = asRecord(currentConfig.value.channels)
  const feishu = asRecord(channels.feishu)
  const dingtalk = asRecord(resolveDingtalkChannelNode(channels))
  const telegram = asRecord(channels.telegram)
  const discord = asRecord(channels.discord)
  const slack = asRecord(channels.slack)
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
  if (dingtalk.clientId && dingtalk.clientSecret) {
    selectedChannelId.value = 'dingtalk'
    channelIdValue.value = String(dingtalk.clientId)
    channelSecretValue.value = String(dingtalk.clientSecret)
    markStepSaved('channel')
    return
  }
  if (telegram.botToken) {
    selectedChannelId.value = 'telegram'
    channelSecretValue.value = String(telegram.botToken)
    markStepSaved('channel')
    return
  }
  if (discord.token) {
    selectedChannelId.value = 'discord'
    channelSecretValue.value = String(discord.token)
    markStepSaved('channel')
    return
  }
  if (slack.botToken) {
    selectedChannelId.value = 'slack'
    channelSecretValue.value = String(slack.botToken)
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

const refreshModels = async () => {
  const apiKey = providerApiKey.value.trim()
  if (!apiKey) {
    errorMessage.value = '填写 API Key 后才能拉取模型列表'
    return
  }
  loadingModels.value = true
  errorMessage.value = ''
  try {
    fetchedModels.value = await invoke<string[]>('fetch_provider_models', {
      baseUrl: currentProviderPreset.value.baseUrl,
      apiKey,
    })
  } catch (error) {
    errorMessage.value = `模型列表拉取失败：${error}`
  } finally {
    loadingModels.value = false
  }
}

const saveModelStep = async () => {
  const apiKey = providerApiKey.value.trim()
  const modelId = modelQuery.value.trim()
  if (!apiKey) throw new Error('请先填写 API Key')
  if (!modelId) throw new Error('请先选择或输入模型 ID')
  if (!currentConfig.value) throw new Error('配置文件未加载')

  setBusy('正在写入模型配置...')
  try {
    const preset = currentProviderPreset.value
    const nextConfig = applyQuickSetupModelPreset(currentConfig.value, preset, apiKey, modelId)

    currentConfig.value = nextConfig
    await saveConfigToDisk()
    await refreshProvidersAndSelection()
    markStepSaved('model')
    infoMessage.value = `主模型已更新为 ${preset.displayName} / ${modelId}`
    props.showToast('success', '模型配置已保存')
    moveNext()
  } finally {
    clearBusy()
  }
}

const ensureSelectedChannelExtension = async () => {
  if (!isExtensionChannel.value || extensionInstalled.value) return
  await invoke<string>('install_channel_extension', { channelId: selectedChannelId.value })
  await refreshChannelExtensionStatus()
}

const ensureGenericChannelNode = (channelId: 'telegram' | 'discord' | 'slack') => {
  if (!currentConfig.value) throw new Error('配置文件未加载')
  const root = asRecord(currentConfig.value)
  const channels = asRecord(root.channels)
  root.channels = channels
  const node = asRecord(channels[channelId])
  channels[channelId] = node
  return node
}

const saveGenericChannel = async () => {
  const secret = channelSecretValue.value.trim()
  const primaryField = channelIdValue.value.trim()
  if (!secret) throw new Error(`${currentChannelPreset.value.secretLabel} 不能为空`)
  if (!currentConfig.value) throw new Error('配置文件未加载')

  if (selectedChannelId.value === 'telegram') {
    const node = ensureGenericChannelNode('telegram')
    node.enabled = true
    node.botToken = secret
    node.dmPolicy ??= 'pairing'
    node.groupPolicy ??= 'allowlist'
    node.replyToMode ??= 'off'
  }

  if (selectedChannelId.value === 'discord') {
    const node = ensureGenericChannelNode('discord')
    node.enabled = true
    node.token = secret
    node.dm ??= { policy: 'pairing' }
    node.groupPolicy ??= 'allowlist'
    node.replyToMode ??= 'off'
  }

  if (selectedChannelId.value === 'slack') {
    const node = ensureGenericChannelNode('slack')
    if (!primaryField) throw new Error('Signing Secret 不能为空')
    node.enabled = true
    node.botToken = secret
    node.signingSecret = primaryField
    node.mode ??= 'http'
    node.webhookPath ??= '/webhooks/slack'
    node.dmPolicy ??= 'pairing'
    node.groupPolicy ??= 'allowlist'
    node.replyToMode ??= 'off'
    node.requireMention ??= true
  }

  await saveConfigToDisk()
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
      await invoke<string>('set_feishu_channel_config', {
        appId,
        appSecret,
        enabled: true,
      })
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
    } else {
      await saveGenericChannel()
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
  clearQuickSetupSession()
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

    markStepSaved('gateway')
    clearQuickSetupSession()
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
  const providerName = currentProviderPreset.value.name
  providerApiKey.value = currentConfig.value?.models?.providers?.[providerName]?.apiKey || ''
  modelQuery.value = currentProviderPreset.value.suggestedModels[0]?.id || ''
})

watch(selectedChannelId, () => {
  channelIdValue.value = ''
  channelSecretValue.value = ''
  errorMessage.value = ''
})

watch(
  [
    stepIndex,
    savedStepIds,
    selectedProviderId,
    providerApiKey,
    modelQuery,
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

      <section class="oc-panel oc-quick-setup-main flex min-h-0 flex-col p-5">
        <div class="flex items-start justify-between gap-4 border-b pb-4" style="border-color: var(--oc-divider-soft);">
          <div>
            <div class="flex items-center gap-2 text-xs font-medium uppercase tracking-[0.18em]" style="color: var(--oc-text-muted);">
              <component :is="currentStep.id === 'model' ? Bot : currentStep.id === 'channel' ? MessageSquareMore : Rocket" class="h-4 w-4" />
              Step {{ stepIndex + 1 }}
            </div>
            <h2 class="mt-2 text-2xl font-semibold" style="color: var(--oc-text-primary);">{{ currentStep.title }}</h2>
            <p class="mt-2 text-sm leading-6" style="color: var(--oc-text-secondary);">{{ currentStep.subtitle }}</p>
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

        <div class="mt-5 flex min-h-0 flex-1 flex-col">
          <div v-if="currentStep.id === 'model'" class="grid min-h-0 flex-1 gap-4 xl:grid-cols-[minmax(0,280px),minmax(0,1fr),minmax(0,1fr)]">
            <div class="rounded-[18px] border p-4" style="border-color: color-mix(in srgb, var(--oc-accent) 10%, var(--oc-card-border)); background: var(--oc-card);">
              <label class="mb-2 block text-sm font-medium" style="color: var(--oc-text-secondary);">预设服务商</label>
              <div class="relative">
                <select
                  v-model="selectedProviderId"
                  class="oc-input w-full appearance-none pr-10"
                  aria-label="选择预设服务商"
                >
                  <option
                    v-for="preset in QUICK_SETUP_PROVIDER_PRESETS"
                    :key="preset.id"
                    :value="preset.id"
                  >
                    {{ preset.displayName }}
                  </option>
                </select>
                <ChevronDown class="pointer-events-none absolute right-3 top-1/2 h-4 w-4 -translate-y-1/2" style="color: var(--oc-text-muted);" />
              </div>
              <p class="mt-3 text-sm font-medium" style="color: var(--oc-text-primary);">{{ currentProviderPreset.displayName }}</p>
              <p class="mt-2 text-xs leading-5" style="color: var(--oc-text-secondary);">{{ currentProviderPreset.description }}</p>
              <p class="mt-3 break-all text-[11px] leading-5" style="color: var(--oc-text-muted);">{{ currentProviderPreset.baseUrl }}</p>
            </div>

            <div class="rounded-[18px] border p-4" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated);">
              <label class="mb-2 block text-sm font-medium" style="color: var(--oc-text-secondary);">API Key</label>
              <Input v-model="providerApiKey" type="password" placeholder="输入服务商 API Key" autocomplete="off" />
              <p class="mt-2 text-xs leading-5" style="color: var(--oc-text-muted);">
                当前配置页中同名服务商的 Key 也会在这里复用：{{ currentProviderApiKey ? '已检测到已有 Key' : '未检测到' }}
              </p>
            </div>

            <div class="rounded-[18px] border p-4" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated);">
              <div class="mb-2 flex items-center justify-between gap-3">
                <label class="block text-sm font-medium" style="color: var(--oc-text-secondary);">模型 ID / 名称</label>
                <button type="button" class="oc-toolbar-btn h-8 px-3 text-xs" :disabled="loadingModels || busy" @click="refreshModels">
                  <Loader2 v-if="loadingModels" class="h-3.5 w-3.5 animate-spin" />
                  <RefreshCw v-else class="h-3.5 w-3.5" />
                  刷新列表
                </button>
              </div>
              <input
                v-model="modelQuery"
                class="oc-input w-full"
                list="quick-setup-model-options"
                placeholder="搜索模型或直接输入模型 ID"
                autocomplete="off"
              />
              <datalist id="quick-setup-model-options">
                <option
                  v-for="item in filteredModelOptions"
                  :key="item.id"
                  :value="item.id"
                  :label="item.name"
                >{{ item.name }}</option>
              </datalist>
              <p class="mt-2 text-xs leading-5" style="color: var(--oc-text-muted);">支持直接输入自定义模型 ID；刷新后可从系统返回列表中快速选择。</p>
            </div>
          </div>

          <div v-else-if="currentStep.id === 'channel'" class="grid min-h-0 flex-1 gap-4 xl:grid-cols-[minmax(0,1.05fr),minmax(0,1fr)]">
            <div class="rounded-[18px] border p-4" style="border-color: color-mix(in srgb, var(--oc-accent) 10%, var(--oc-card-border)); background: var(--oc-card);">
              <p class="mb-3 text-sm font-medium" style="color: var(--oc-text-primary);">选择通信渠道</p>
              <div class="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
                <button
                  v-for="channel in QUICK_SETUP_CHANNEL_PRESETS"
                  :key="channel.id"
                  type="button"
                  class="rounded-[16px] border p-4 text-left transition-all"
                  :style="{
                    borderColor: selectedChannelId === channel.id ? 'var(--oc-card-border-strong)' : 'var(--oc-card-border)',
                    background: selectedChannelId === channel.id ? 'var(--oc-item-active)' : 'var(--oc-card-elevated)',
                  }"
                  @click="selectedChannelId = channel.id"
                >
                  <div class="flex items-center justify-between gap-3">
                    <p class="text-sm font-semibold" style="color: var(--oc-text-primary);">{{ channel.name }}</p>
                    <span v-if="channel.id === 'feishu' || channel.id === 'dingtalk'" class="rounded-full px-2 py-0.5 text-[11px]" style="background: color-mix(in srgb, var(--oc-card) 88%, transparent); color: var(--oc-text-muted);">
                      {{ channel.id === 'feishu' ? (channelExtensionStatus.feishuInstalled ? '已安装扩展' : '待安装扩展') : (channelExtensionStatus.dingtalkInstalled ? '已安装扩展' : '待安装扩展') }}
                    </span>
                  </div>
                  <p class="mt-2 text-xs leading-5" style="color: var(--oc-text-secondary);">{{ channel.description }}</p>
                </button>
              </div>
            </div>

            <div class="grid gap-4 lg:grid-rows-2">
              <div class="rounded-[16px] border p-4" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated);">
                <label class="mb-2 block text-sm font-medium" style="color: var(--oc-text-secondary);">{{ currentChannelPreset.placeholderLabel }}</label>
                <Input
                  v-model="channelIdValue"
                  :disabled="!channelNeedsPrimaryField"
                  :placeholder="channelNeedsPrimaryField ? currentChannelPreset.placeholderLabel : '该渠道无需填写此项'"
                />
                <p class="mt-2 text-xs" style="color: var(--oc-text-muted);">{{ currentChannelPreset.description }}</p>
              </div>

              <div class="rounded-[16px] border p-4" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated);">
                <label class="mb-2 block text-sm font-medium" style="color: var(--oc-text-secondary);">{{ currentChannelPreset.secretLabel }}</label>
                <Input v-model="channelSecretValue" type="password" :placeholder="`输入${currentChannelPreset.secretLabel}`" autocomplete="off" />
                <p class="mt-2 text-xs" style="color: var(--oc-text-muted);">仅写入当前选择且已填写的渠道配置。</p>
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

        <div class="mt-5 flex items-center justify-between gap-3 border-t pt-4" style="border-color: var(--oc-divider-soft);">
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
            <Button v-if="currentStep.id === 'model'" variant="outline" :disabled="busy || loadingModels" @click="refreshModels">
              <RefreshCw class="h-4 w-4" />
              获取模型
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
