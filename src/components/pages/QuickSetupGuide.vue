<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import {
  Bot,
  CheckCircle2,
  ChevronLeft,
  Loader2,
  MessageSquareMore,
  RefreshCw,
  Rocket,
  ShieldCheck,
  Sparkles,
} from 'lucide-vue-next'
import Button from '../ui/Button.vue'
import Input from '../ui/Input.vue'
import { isPrimaryModelPlaceholder } from '../../domain/configValidation'
import { waitForGatewayReady } from '../../domain/gatewayStartup'
import {
  QUICK_SETUP_CHANNEL_PRESETS,
  QUICK_SETUP_PROVIDER_PRESETS,
  QUICK_SETUP_STEPS,
  canSkipQuickSetupStep,
  findProviderPreset,
  getGatewayInstallPlan,
  type QuickSetupChannelId,
  type QuickSetupProviderId,
  type QuickSetupStepId,
} from '../../domain/quickSetupGuide'
import { resolveDingtalkChannelNode } from '../../domain/dingtalkPlugin'
import type { ConfigFileInfo, ModelSelectionInfo, OpenClawConfig, ProviderInfo } from '../../types/config'

const props = defineProps<{
  showToast: (type: 'success' | 'error', message: string) => void
  systemOs: 'windows' | 'macos' | 'linux'
}>()

const emit = defineEmits<{
  complete: []
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
const showModelDropdown = ref(false)

const selectedChannelId = ref<QuickSetupChannelId>('feishu')
const channelIdValue = ref('')
const channelSecretValue = ref('')

const savedStepIds = ref<QuickSetupStepId[]>([])

const currentStep = computed(() => QUICK_SETUP_STEPS[stepIndex.value])
const currentProviderPreset = computed(() => findProviderPreset(selectedProviderId.value)!)
const currentChannelPreset = computed(
  () => QUICK_SETUP_CHANNEL_PRESETS.find((item) => item.id === selectedChannelId.value) ?? QUICK_SETUP_CHANNEL_PRESETS[0]
)
const gatewayPlan = computed(() => getGatewayInstallPlan(props.systemOs))
const hasReadyPrimaryModel = computed(() => {
  const primary = modelSelection.value.primary
  return Boolean(primary && !isPrimaryModelPlaceholder(primary))
})
const isExtensionChannel = computed(() => selectedChannelId.value === 'feishu' || selectedChannelId.value === 'dingtalk')
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
  return maskSecret(channelSecretValue.value)
})

const asRecord = (value: unknown): Record<string, any> =>
  value && typeof value === 'object' && !Array.isArray(value) ? (value as Record<string, any>) : {}

const markStepSaved = (stepId: QuickSetupStepId) => {
  if (!savedStepIds.value.includes(stepId)) {
    savedStepIds.value = [...savedStepIds.value, stepId]
  }
}

const setBusy = (message: string) => {
  busy.value = true
  busyMessage.value = message
  errorMessage.value = ''
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
    showModelDropdown.value = true
  } catch (error) {
    errorMessage.value = `模型列表拉取失败：${error}`
  } finally {
    loadingModels.value = false
  }
}

const selectModelOption = (modelId: string) => {
  modelQuery.value = modelId
  showModelDropdown.value = false
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
    let nextConfig = await invoke<OpenClawConfig>('upsert_provider', {
      config: currentConfig.value,
      name: preset.name,
      baseUrl: preset.baseUrl,
      apiKey,
      api: null,
    })

    const provider = nextConfig.models?.providers?.[preset.name]
    const modelExists = provider?.models?.some((item) => item.id === modelId)
    if (!modelExists) {
      nextConfig = await invoke<OpenClawConfig>('add_model_to_provider', {
        config: nextConfig,
        providerName: preset.name,
        modelId,
        modelName: null,
      })
    }

    nextConfig = await invoke<OpenClawConfig>('set_primary_model', {
      config: nextConfig,
      modelPath: `${preset.name}/${modelId}`,
    })

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
    node.enabled = true
    node.botToken = secret
    node.mode ??= 'http'
    node.webhookPath ??= '/webhooks/slack'
    node.dmPolicy ??= 'pairing'
    node.groupPolicy ??= 'allowlist'
    node.replyToMode ??= 'off'
    node.requireMention ??= true
  }

  await saveConfigToDisk()
}

const saveChannelStep = async () => {
  setBusy('正在写入通信渠道配置...')
  try {
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

const installGatewayAndEnterDashboard = async () => {
  if (!hasReadyPrimaryModel.value) {
    stepIndex.value = 0
    throw new Error('进入工作台前需要先完成有效主模型配置')
  }

  setBusy('正在安装并启动网关...')
  try {
    await invoke<string>('install_gateway_service')
    if (props.systemOs !== 'windows') {
      setBusy('正在启动网关服务...')
      await invoke<string>('start_gateway')
    }
    setBusy('正在等待网关健康检查通过...')
    const ready = await waitForGatewayReady(() => invoke<boolean>('health_check_gateway'), {
      maxAttempts: 24,
      intervalMs: 1500,
    })
    if (!ready) {
      throw new Error('网关在预期时间内未完成启动，请稍后重试')
    }

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
    errorMessage.value = String(error)
    props.showToast('error', String(error))
  }
}

watch(selectedProviderId, () => {
  fetchedModels.value = []
  showModelDropdown.value = false
  const providerName = currentProviderPreset.value.name
  if (currentConfig.value?.models?.providers?.[providerName]?.apiKey) {
    providerApiKey.value = currentConfig.value.models.providers[providerName]?.apiKey || ''
  }
  if (!modelQuery.value) {
    modelQuery.value = currentProviderPreset.value.suggestedModels[0]?.id || ''
  }
})

watch(selectedChannelId, () => {
  channelIdValue.value = ''
  channelSecretValue.value = ''
  errorMessage.value = ''
})

onMounted(async () => {
  setBusy('正在准备快速引导...')
  try {
    await ensureDefaultConfigReady()
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
      <aside class="oc-panel flex min-h-0 flex-col p-4">
        <div class="rounded-[18px] border p-4" style="border-color: var(--oc-card-border); background: linear-gradient(135deg, color-mix(in srgb, var(--oc-accent-soft) 72%, var(--oc-card) 28%), var(--oc-card));">
          <div class="flex items-center gap-3">
            <div class="flex h-11 w-11 items-center justify-center rounded-full" style="background: color-mix(in srgb, var(--oc-accent) 18%, transparent); color: var(--oc-accent);">
              <Sparkles class="h-5 w-5" />
            </div>
            <div>
              <p class="text-base font-semibold" style="color: var(--oc-text-primary);">快速引导</p>
              <p class="text-xs" style="color: var(--oc-text-secondary);">统一安装后门禁与首次配置入口</p>
            </div>
          </div>
          <p class="mt-3 text-sm leading-6" style="color: var(--oc-text-secondary);">
            先写入默认配置，再用最少步骤完成模型、通信渠道与网关安装，所有内容都会同步到正式配置页。
          </p>
        </div>

        <div class="mt-4 space-y-2">
          <button
            v-for="(step, index) in QUICK_SETUP_STEPS"
            :key="step.id"
            type="button"
            class="w-full rounded-[14px] border px-4 py-3 text-left transition-all"
            :style="{
              borderColor: index === stepIndex ? 'var(--oc-card-border-strong)' : 'var(--oc-card-border)',
              background: index === stepIndex ? 'var(--oc-item-active)' : 'var(--oc-card-elevated)',
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

        <div class="mt-auto rounded-[14px] border p-3 text-xs leading-5" style="border-color: var(--oc-divider); background: color-mix(in srgb, var(--oc-card-elevated) 86%, transparent); color: var(--oc-text-secondary);">
          <p>当前主模型：<span style="color: var(--oc-text-primary);">{{ primaryModelPath || '未设置' }}</span></p>
          <p class="mt-1">当前渠道：<span style="color: var(--oc-text-primary);">{{ currentChannelPreset.name }}</span></p>
          <p class="mt-1">当前配置文件：<span style="color: var(--oc-text-primary);">{{ fileInfo?.fileName || 'openclaw.json' }}</span></p>
        </div>
      </aside>

      <section class="oc-panel flex min-h-0 flex-col p-5">
        <div class="flex items-start justify-between gap-4 border-b pb-4" style="border-color: var(--oc-divider-soft);">
          <div>
            <div class="flex items-center gap-2 text-xs font-medium uppercase tracking-[0.18em]" style="color: var(--oc-text-muted);">
              <component :is="currentStep.id === 'model' ? Bot : currentStep.id === 'channel' ? MessageSquareMore : Rocket" class="h-4 w-4" />
              Step {{ stepIndex + 1 }}
            </div>
            <h2 class="mt-2 text-2xl font-semibold" style="color: var(--oc-text-primary);">{{ currentStep.title }}</h2>
            <p class="mt-2 text-sm leading-6" style="color: var(--oc-text-secondary);">{{ currentStep.subtitle }}</p>
          </div>
          <div class="rounded-full px-3 py-1 text-xs font-medium" style="background: color-mix(in srgb, var(--oc-card-elevated) 88%, transparent); color: var(--oc-text-secondary);">
            跳过仅支持前两步
          </div>
        </div>

        <div v-if="errorMessage" class="mt-4 rounded-[14px] border px-4 py-3 text-sm" style="border-color: color-mix(in srgb, var(--oc-danger) 32%, transparent); background: color-mix(in srgb, var(--oc-danger) 9%, transparent); color: var(--oc-danger);">
          {{ errorMessage }}
        </div>
        <div v-else-if="infoMessage || busyMessage" class="mt-4 rounded-[14px] border px-4 py-3 text-sm" style="border-color: color-mix(in srgb, var(--oc-accent) 28%, transparent); background: color-mix(in srgb, var(--oc-accent) 10%, transparent); color: var(--oc-text-secondary);">
          <span v-if="busy" class="inline-flex items-center gap-2"><Loader2 class="h-4 w-4 animate-spin" />{{ busyMessage }}</span>
          <span v-else>{{ infoMessage }}</span>
        </div>

        <div class="mt-5 min-h-0 flex-1 overflow-auto pr-1">
          <div v-if="currentStep.id === 'model'" class="space-y-5">
            <div>
              <p class="mb-3 text-sm font-medium" style="color: var(--oc-text-primary);">选择服务商预设</p>
              <div class="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
                <button
                  v-for="preset in QUICK_SETUP_PROVIDER_PRESETS"
                  :key="preset.id"
                  type="button"
                  class="rounded-[16px] border p-4 text-left transition-all"
                  :style="{
                    borderColor: selectedProviderId === preset.id ? 'var(--oc-card-border-strong)' : 'var(--oc-card-border)',
                    background: selectedProviderId === preset.id ? 'var(--oc-item-active)' : 'var(--oc-card-elevated)',
                  }"
                  @click="selectedProviderId = preset.id"
                >
                  <div class="flex items-center justify-between gap-3">
                    <p class="text-sm font-semibold" style="color: var(--oc-text-primary);">{{ preset.displayName }}</p>
                    <ShieldCheck class="h-4 w-4" :style="{ color: selectedProviderId === preset.id ? 'var(--oc-accent)' : 'var(--oc-text-quiet)' }" />
                  </div>
                  <p class="mt-2 text-xs leading-5" style="color: var(--oc-text-secondary);">{{ preset.description }}</p>
                  <p class="mt-3 text-[11px] leading-5" style="color: var(--oc-text-muted);">{{ preset.baseUrl }}</p>
                </button>
              </div>
            </div>

            <div class="grid gap-4 lg:grid-cols-[minmax(0,1fr),minmax(0,1fr)]">
              <div class="rounded-[16px] border p-4" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated);">
                <label class="mb-2 block text-sm font-medium" style="color: var(--oc-text-secondary);">API Key</label>
                <Input v-model="providerApiKey" type="password" placeholder="输入服务商 API Key" autocomplete="off" />
                <p class="mt-2 text-xs leading-5" style="color: var(--oc-text-muted);">
                  当前配置页中同名服务商的 Key 也会在这里复用：{{ currentProviderApiKey ? '已检测到已有 Key' : '未检测到' }}
                </p>
              </div>

              <div class="rounded-[16px] border p-4" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated);">
                <div class="mb-2 flex items-center justify-between gap-3">
                  <label class="block text-sm font-medium" style="color: var(--oc-text-secondary);">模型 ID / 名称</label>
                  <button type="button" class="oc-toolbar-btn h-8 px-3 text-xs" :disabled="loadingModels || busy" @click="refreshModels">
                    <Loader2 v-if="loadingModels" class="h-3.5 w-3.5 animate-spin" />
                    <RefreshCw v-else class="h-3.5 w-3.5" />
                    刷新列表
                  </button>
                </div>
                <div class="relative">
                  <Input v-model="modelQuery" placeholder="搜索模型或直接输入模型 ID" @focus="showModelDropdown = true" />
                  <div v-if="showModelDropdown && filteredModelOptions.length > 0" class="absolute z-20 mt-2 max-h-56 w-full overflow-auto rounded-[14px] border p-2 shadow-lg" style="border-color: var(--oc-card-border); background: var(--oc-card); box-shadow: var(--oc-shadow-popover);">
                    <button
                      v-for="item in filteredModelOptions"
                      :key="item.id"
                      type="button"
                      class="w-full rounded-[10px] px-3 py-2 text-left text-sm transition-colors"
                      style="color: var(--oc-text-secondary);"
                      @click="selectModelOption(item.id)"
                    >
                      <div class="font-medium" style="color: var(--oc-text-primary);">{{ item.name }}</div>
                      <div class="text-xs" style="color: var(--oc-text-muted);">{{ item.id }}</div>
                    </button>
                  </div>
                </div>
                <div class="mt-3 flex flex-wrap gap-2">
                  <button
                    v-for="item in currentProviderPreset.suggestedModels"
                    :key="item.id"
                    type="button"
                    class="rounded-full border px-3 py-1.5 text-xs transition-colors"
                    :style="{
                      borderColor: modelQuery === item.id ? 'var(--oc-card-border-strong)' : 'var(--oc-card-border)',
                      background: modelQuery === item.id ? 'var(--oc-item-active)' : 'transparent',
                      color: modelQuery === item.id ? 'var(--oc-text-primary)' : 'var(--oc-text-secondary)',
                    }"
                    @click="modelQuery = item.id"
                  >
                    {{ item.name }}
                  </button>
                </div>
              </div>
            </div>
          </div>

          <div v-else-if="currentStep.id === 'channel'" class="space-y-5">
            <div>
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

            <div class="grid gap-4 lg:grid-cols-2">
              <div class="rounded-[16px] border p-4" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated);">
                <label class="mb-2 block text-sm font-medium" style="color: var(--oc-text-secondary);">{{ currentChannelPreset.placeholderLabel }}</label>
                <Input
                  v-model="channelIdValue"
                  :disabled="!isExtensionChannel"
                  :placeholder="isExtensionChannel ? currentChannelPreset.placeholderLabel : '该渠道快速模式下无需填写此项'"
                />
                <p class="mt-2 text-xs" style="color: var(--oc-text-muted);">
                  {{ isExtensionChannel ? '飞书/钉钉仅需填写 ID 与 Key，扩展缺失时会自动安装。' : 'Telegram / Discord / Slack 只保留 token 快速接入。' }}
                </p>
              </div>

              <div class="rounded-[16px] border p-4" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated);">
                <label class="mb-2 block text-sm font-medium" style="color: var(--oc-text-secondary);">{{ currentChannelPreset.secretLabel }}</label>
                <Input v-model="channelSecretValue" type="password" :placeholder="`输入${currentChannelPreset.secretLabel}`" autocomplete="off" />
                <p class="mt-2 text-xs" style="color: var(--oc-text-muted);">
                  保存后默认启用渠道，其余参数保留最精简默认值，并同步到正式通信渠道页。
                </p>
              </div>
            </div>
          </div>

          <div v-else class="space-y-5">
            <div class="grid gap-4 lg:grid-cols-[minmax(0,1fr),minmax(0,1fr)]">
              <div class="rounded-[16px] border p-4" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated);">
                <p class="text-sm font-semibold" style="color: var(--oc-text-primary);">当前配置摘要</p>
                <div class="mt-3 space-y-2 text-sm" style="color: var(--oc-text-secondary);">
                  <p>主模型：<span style="color: var(--oc-text-primary);">{{ primaryModelPath || '未配置' }}</span></p>
                  <p>通信渠道：<span style="color: var(--oc-text-primary);">{{ currentChannelPreset.name }}</span></p>
                  <p>渠道凭据：<span style="color: var(--oc-text-primary);">{{ selectedChannelSummary || '未配置' }}</span></p>
                </div>
              </div>

              <div class="rounded-[16px] border p-4" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated);">
                <p class="text-sm font-semibold" style="color: var(--oc-text-primary);">当前系统执行计划</p>
                <p class="mt-2 text-sm" style="color: var(--oc-text-secondary);">{{ gatewayPlan.summary }}</p>
                <div class="mt-3 flex flex-wrap gap-2">
                  <span
                    v-for="command in gatewayPlan.commands"
                    :key="command"
                    class="rounded-full border px-3 py-1.5 text-xs"
                    style="border-color: var(--oc-card-border); background: color-mix(in srgb, var(--oc-card) 88%, transparent); color: var(--oc-text-secondary);"
                  >
                    {{ command }}
                  </span>
                </div>
              </div>
            </div>

            <div class="rounded-[16px] border p-4 text-sm leading-6" style="border-color: var(--oc-card-border); background: color-mix(in srgb, var(--oc-card-elevated) 88%, transparent); color: var(--oc-text-secondary);">
              <p>网关安装完成后会自动等待健康检查通过，再进入工作台页面。</p>
              <p class="mt-2">如果你前面跳过了模型步骤，这里会阻止进入工作台，并提示先完成有效主模型配置。</p>
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
          </div>
        </div>
      </section>
    </div>
  </div>
</template>


